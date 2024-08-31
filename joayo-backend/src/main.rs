use std::sync::Arc;

use bytes::Bytes;
use tracing::{error, warn};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tokio::{select, signal::unix::{signal, SignalKind}, sync::watch, task};

#[tokio::main(flavor="multi_thread")]
async fn main() {
    //Channel to exchange graceful shutdown request
    let (api_shutdown_tx, api_shutdown_rx) = watch::channel(());
    let (service_shutdown_tx, service_shutdown_rx) = watch::channel(());

    //Channel to exchange image encode & upload request
    let (encode_tx, encode_rx) = crossbeam::channel::unbounded::<Arc<Bytes>>();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
            .with_filter(filter::LevelFilter::INFO)
        )
        .init();

    let axum = task::spawn(api::axum_start(api_shutdown_rx, encode_tx));
    let service = task::spawn(service::queue_executor(service_shutdown_rx, encode_rx));
    let shutdown = task::spawn(async {
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
            break;
        };
    });

    shutdown.await.unwrap();
    service_shutdown_tx.send(()).unwrap();
    api_shutdown_tx.send(()).unwrap();

    service.await.unwrap();
    axum.await.unwrap();
}
