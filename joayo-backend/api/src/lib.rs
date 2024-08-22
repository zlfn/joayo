use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use migration::Migrator;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm_migration::MigratorTrait;
use serde::Deserialize;
use entities::{prelude::*, *};
use uuid::Uuid;

pub mod migration;
pub mod entities;

#[derive(Clone)]
struct AppState {
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
        .route("/register", post(create_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7878").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CreateUser {
    email: String,
    password: String,
}

async fn create_user(
    state: State<AppState>,
    Json(payload): Json<CreateUser>,
) -> StatusCode {

    let user_with_email = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db).await;

    match user_with_email {
        Ok(None) => (),
        Ok(_) => return StatusCode::CONFLICT,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR
    }

    let new_user = user::ActiveModel {
        user_id: ActiveValue::Set(Uuid::new_v4()),
        email: ActiveValue::Set(payload.email.clone()),
    };

    let user_res = User::insert(new_user).exec(&state.db).await;
    let user_res = match user_res {
        Ok(res) => res,
        Err(_) => {
            return StatusCode::INTERNAL_SERVER_ERROR
        }
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(&payload.password.clone().into_bytes(), &salt).unwrap();

    let new_password = password_identity::ActiveModel {
        user_id: ActiveValue::Set(user_res.last_insert_id),
        password_hash: ActiveValue::Set(hashed_password.to_string()),
        salt: ActiveValue::Set(salt.to_string())
    };

    let pw_res = PasswordIdentity::insert(new_password).exec(&state.db).await;
    if let Err(_) = pw_res {
        //If failed to create password, try delete user
        let user_to_delete = user::ActiveModel {
            user_id: ActiveValue::Set(user_res.last_insert_id),
            ..Default::default()
        };
        user_to_delete.delete(&state.db).await.unwrap();
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::CREATED
}

async fn root() -> &'static str {
    "Welcome to JOAYO API!"
}

