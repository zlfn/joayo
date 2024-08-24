use axum::{routing::{get, post}, Router};
use migration::Migrator;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tracing::info;
use tracing_subscriber::{filter, prelude::*, layer::SubscriberExt};

pub mod migration;
pub mod entities;
pub mod user;
pub mod server_result;

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
}

#[tokio::main]
pub async fn start() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().pretty()
            .with_filter(filter::LevelFilter::WARN)
        )
        .init();

    let db: DatabaseConnection = Database::connect("postgres://joayo:joayo@0.0.0.0/joayo")
        .await
        .expect("Failed to connect database");

    info!("Database connection established.");

    Migrator::up(&db, None).await.unwrap();
    //Migrator::down(&db, None).await.unwrap();

    let state = AppState { db };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(user::create_user))
        .route("/login", post(user::get_session))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn root() -> &'static str {
    "Hello, JOAYO API!"
}
