use axum::{routing::{get, post}, Router};
use migration::Migrator;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

pub mod migration;
pub mod entities;
pub mod user;

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
}

#[tokio::main]
pub async fn start() {
    tracing_subscriber::fmt::init();


    let db: DatabaseConnection = Database::connect("postgres://joayo:joayo@0.0.0.0/joayo")
        .await
        .expect("Failed to connect database");

    Migrator::up(&db, None).await.unwrap();

    let state = AppState { db };

    let app = Router::new()
        .route("/", get(root))
        .route("/register", post(user::create_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


async fn root() -> &'static str {
    "Welcome to JOAYO API!"
}

