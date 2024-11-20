use std::{
    collections::HashMap,
    error::Error,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use askama::Template;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::{Message, WebSocket};

use crate::{templating::Page, worker::State};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Default)]
pub struct WebsocketLobby {
    connections: RwLock<HashMap<usize, mpsc::UnboundedSender<Message>>>,
}

impl WebsocketLobby {
    pub async fn on_connect(self: Arc<Self>, ws: WebSocket) {
        let (mut user_ws_tx, mut user_ws_rx) = ws.split();

        let (tx, rx) = mpsc::unbounded_channel();
        let mut rx = UnboundedReceiverStream::new(rx);

        tokio::task::spawn(async move {
            while let Some(message) = rx.next().await {
                user_ws_tx
                    .send(message)
                    .unwrap_or_else(|e| {
                        eprintln!("> websocket send error: {}", e);
                    })
                    .await;
            }
        });

        let id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

        self.connections.write().await.insert(id, tx);

        while let Some(result) = user_ws_rx.next().await {
            match result {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("> websocket error(uid={}): {}", id, e);
                    break;
                }
            };
        }

        self.on_disconnect(id).await;
    }

    async fn on_disconnect(&self, id: usize) {
        self.connections.write().await.remove(&id);
    }

    pub async fn broadcast(self: Arc<Self>, msg: Message) -> Result<(), Box<dyn Error>> {
        let r_lock = self.connections.read().await;
        if r_lock.len() <= 0 {
            return Ok(());
        }

        for (_id, conn) in r_lock.iter() {
            if let Err(e) = conn.send(msg.clone()) {
                eprintln!("> Failed to send update to connection: {e}");
            }
        }

        Ok(())
    }
}

pub async fn handle_lobby(
    state: Arc<RwLock<State>>,
    lobby: Arc<WebsocketLobby>,
) -> Result<(), Box<dyn Error>> {
    let r_lock = state.read().await;
    let template = Page::new(&r_lock);
    let html = template.render()?;
    drop(r_lock);

    lobby.broadcast(Message::text(html)).await?;

    Ok(())
}
