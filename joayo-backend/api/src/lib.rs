use std::time::Duration;

use axum::{extract::DefaultBodyLimit, routing::{delete, get, post, put}, Router};
use migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tokio::{select, sync::watch::Receiver};
use tower_http::timeout::TimeoutLayer;
use tracing::info;

pub mod migration;
pub mod entities;
pub mod user;
pub mod joayo;
pub mod common;
pub mod server_result;

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
}

pub async fn axum_start(mut shutdown_rx: Receiver<()>) {

    let mut opt = ConnectOptions::new("postgres://joayo:joayo@0.0.0.0/joayo".to_owned());
    opt.sqlx_logging_level(log::LevelFilter::Debug);
    let db: DatabaseConnection = Database::connect(opt)
        .await
        .expect("Failed to connect database");

    info!("Database connection established.");

    Migrator::up(&db, None).await.unwrap();
    //Migrator::down(&db, None).await.unwrap();

    let state = AppState { db: db.clone() };

    let app = Router::new()
        .route("/", get(root))
        .route("/user", post(user::create_user))
        .route("/password", put(user::change_password))
        .route("/session", get(user::check_session))
        .route("/session", post(user::get_session))
        .route("/session", delete(user::delete_session))
        .route("/joayo", post(joayo::create_joayo))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(DefaultBodyLimit::max(50*1024*1024)) //50MB
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            select! {
                _ = shutdown_rx.changed() => {
                    info!("Shutting down Axum");
                    db.close().await.unwrap();
                    info!("Database connection closed.");
                }
            }
        })
        .await
        .unwrap();
}


async fn root() -> &'static str {
    "Hello, JOAYO API!"
}
