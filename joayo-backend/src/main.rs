use config::Config;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use service::image::ImageUploadRequest;
use tracing::{warn, info};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
use tokio::{select, signal::unix::{signal, SignalKind}, sync::watch, task};

#[tokio::main(flavor="multi_thread")]
async fn main() {
    //Channel to exchange graceful shutdown request
    let (api_shutdown_tx, api_shutdown_rx) = watch::channel(());
    let (service_shutdown_tx, service_shutdown_rx) = watch::channel(());

    //Channel to exchange image encode & upload request
    let (encode_tx, encode_rx) = crossbeam::channel::unbounded::<ImageUploadRequest>();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
            .with_filter(filter::LevelFilter::INFO)
        )
        .init();

    let configs = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    let mut opt = ConnectOptions::new(configs.get_string("database.connection").unwrap());
    opt.sqlx_logging_level(log::LevelFilter::Debug);
    let db: DatabaseConnection = Database::connect(opt)
        .await
        .expect("Failed to connect database");

    info!("Database connection established.");
    orm::migration::Migrator::up(&db, None).await.unwrap();
    //Migrator::down(&db, None).await.unwrap();

    let axum = task::spawn(api::axum_start(db.clone(), api_shutdown_rx, encode_tx));
    let service = task::spawn(service::queue_executor(configs.clone(), service_shutdown_rx, encode_rx));
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

    db.close().await.unwrap();
    info!("Database connection closed.");
}
