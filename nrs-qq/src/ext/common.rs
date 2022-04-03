use alloc::sync::Arc;
use core::sync::atomic::Ordering;

use crate::provider::*;
use crate::Client;

pub async fn after_login<P>(client: &Arc<Client<P>>)
where
    P: ClientProvider + 'static,
{
    if let Err(err) = client.register_client().await {
        tracing::error!("failed to register client: {}", err)
    }
    start_heartbeat(client.clone()).await;
    if let Err(err) = client.refresh_status().await {
        tracing::error!("failed to refresh status: {}", err)
    }
}

pub async fn start_heartbeat<P>(client: Arc<Client<P>>)
where
    P: ClientProvider + 'static,
{
    if !client.heartbeat_enabled.load(Ordering::Relaxed) {
        // tokio::spawn(async move {
        //     client.do_heartbeat().await;
        // });
        P::TP::spawn(async move {
            client.do_heartbeat().await;
        })
        .detach();
    }
}
