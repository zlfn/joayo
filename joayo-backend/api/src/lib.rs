use std::time::Duration;

use axum::{extract::DefaultBodyLimit, routing::{delete, get, post, put}, Router};
use sea_orm::DatabaseConnection;
use service::image::ImageUploadRequest;
use tokio::{select, sync::watch};
use tower_http::timeout::TimeoutLayer;
use tracing::info;

pub mod user;
pub mod joayo;
pub mod common;

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
    image_tx: crossbeam::channel::Sender<ImageUploadRequest>
}

pub async fn axum_start(
    db: DatabaseConnection,
    mut shutdown_rx: watch::Receiver<()>,
    image_tx: crossbeam::channel::Sender<ImageUploadRequest>
 ) {


    let state = AppState { 
        db: db.clone(),
        image_tx: image_tx.clone(),
    };

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
                    info!("Shutting down Axum...");
                }
            }
        })
        .await
        .unwrap();
}


async fn root() -> &'static str {
    "Hello, JOAYO API!"
}
