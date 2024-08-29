use tracing::{error, warn};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tokio::{select, signal::unix::{signal, SignalKind}, sync::watch, task};

#[tokio::main]
async fn main() {
    let (api_shutdown_tx, api_shutdown_rx) = watch::channel(());
    let (cron_shutdown_tx, cron_shutdown_rx) = watch::channel(());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
            .with_filter(filter::LevelFilter::INFO)
        )
        .init();

    let axum = task::spawn(api::axum_start(api_shutdown_rx));
    let cron = task::spawn(cron::queue_executor(cron_shutdown_rx));
    let shutdown = task::spawn(async move {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        loop {
            select! {
                _ = sigterm.recv() => {
                    warn!("SIGTERM detected");
                },
                _ = sigint.recv() => {
                    warn!("SIGINT detected");
                }
            };

            api_shutdown_tx.send(()).unwrap();
            cron_shutdown_tx.send(()).unwrap();
            break;
        };
    });

    if let Err(err) = axum.await {
        error!("{:?}", err);
    }

    if let Err(err) = cron.await {
        error!("{:?}", err);
    }

    if let Err(err) = shutdown.await {
        error!("{:?}", err);
    }
}
