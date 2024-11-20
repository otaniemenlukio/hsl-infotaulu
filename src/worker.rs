use std::{error::Error, sync::Arc};

use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    hsl::{update_state, ApiClient, HslResult},
    ws::{self, WebsocketLobby},
};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub current: HslResult,
}

pub async fn runtime(
    state: Arc<RwLock<State>>,
    client: ApiClient,
    lobby: Arc<WebsocketLobby>,
) -> Result<(), Box<dyn Error>> {
    let sched = JobScheduler::new().await?;

    println!("> Initializing state");
    let _state = state.clone();
    let _client = client.clone();
    update_state(_state, _client).await?;

    let _state = state.clone();
    let _client = client.clone();
    sched
        .add(Job::new_async("1/7 * * * * *", move |_uuid, _l| {
            let s_ref = _state.clone();
            let c_ref = _client.clone();
            Box::pin(async move {
                if let Err(e) = update_state(s_ref, c_ref).await {
                    eprintln!("> Failed to update state: {e}");
                }
            })
        })?)
        .await?;

    let _state = state.clone();
    sched
        .add(Job::new_async("1/1 * * * * *", move |_uuid, _l| {
            let s_ref = _state.clone();
            let l_ref = lobby.clone();
            Box::pin(async move {
                if let Err(e) = ws::handle_lobby(s_ref, l_ref).await {
                    eprintln!("> Failed handle lobby: {e}");
                }
            })
        })?)
        .await?;

    sched.start().await?;

    Ok(())
}
