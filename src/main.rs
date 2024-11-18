use std::{env, error::Error, sync::Arc};

use hsl::create_client;
use tokio::sync::RwLock;
use warp::Filter;
use worker::State;
use ws::WebsocketLobby;

pub mod dates;
pub mod hsl;
pub mod templating;
pub mod worker;
pub mod ws;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();

    let api_key = env::var("API_KEY").expect("Missing API_KEY in .env file");
    let static_dir = env::var("STATIC_DIR").expect("Missing STATIC_DIR in .env file");

    let client = create_client(api_key).await?;
    let state = Arc::new(RwLock::new(State::default()));
    let lobby = Arc::new(WebsocketLobby::default());

    worker::runtime(state, client, lobby.clone()).await?;

    let lobby = warp::any().map(move || lobby.clone());

    let connect = warp::path("connect").and(warp::ws()).and(lobby).map(
        |ws: warp::ws::Ws, lobby: Arc<WebsocketLobby>| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| lobby.on_connect(socket))
        },
    );

    let index = warp::path::end().and(warp::fs::file(format!("{static_dir}/index.html")));
    let static_dir = warp::path("static").and(warp::fs::dir(static_dir));

    let routes = index.or(connect).or(static_dir);

    println!("> Started server on localhost:3060");

    warp::serve(routes).run(([127, 0, 0, 1], 3060)).await;
    Ok(())
}
