use std::time::Duration;

use tokio::{select, sync::watch::Receiver, time};
use tracing::info;

pub async fn queue_executor(mut shutdown_rx: Receiver<()>) {
    loop {
        select! {
            biased;
            _ = shutdown_rx.changed() => {
                info!("Shutdown Service");
                break;
            }
            _i = execute_queue() => (),
        }
    }
}

async fn execute_queue() {
    info!("Queue Executing");
    time::sleep(Duration::from_secs(3)).await;
}
