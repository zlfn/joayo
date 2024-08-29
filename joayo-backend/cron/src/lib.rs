use std::time::Duration;

use tokio::{select, sync::watch::Receiver, time};
use tracing::{debug, info};

pub async fn queue_executor(mut shutdown_rx: Receiver<()>) {
    loop {
        select! {
            biased;
            _ = shutdown_rx.changed() => {
                info!("Shutting down Cron");
                break;
            }
            _i = execute_queue() => (),
        }
    }
}

async fn execute_queue() {
    debug!("Queue Executing");
    time::sleep(Duration::from_secs(5)).await;
}
