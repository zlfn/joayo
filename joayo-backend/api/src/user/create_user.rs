use std::time::Duration;

use api_derive::ToStatusCode;
use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
use axum::{extract::State, http::StatusCode};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::{error, warn};
use uuid::Uuid;


use crate::entities::{prelude::*, *};
use crate::server_result::{ServerResult, ToStatusCode, Json};

#[derive(Deserialize)]
pub struct CreateUserRequest {
    email: String,
    password: String,
}

#[derive(Serialize, ToStatusCode)]
#[status_code(CREATED)]
pub struct CreateUserResponse;

#[derive(Serialize)]
pub enum CreateUserError {
    EmailExist,
    BadPassword,
    InternalServerError,
}

impl ToStatusCode for CreateUserError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::EmailExist => return StatusCode::CONFLICT,
            Self::BadPassword => return StatusCode::UNPROCESSABLE_ENTITY,
            Self::InternalServerError => return StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


pub async fn create_user(
    state: State<crate::AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> ServerResult<CreateUserResponse, CreateUserError> {

    if payload.password.as_bytes().len() < 8 || payload.password.as_bytes().len() > 128 {
        warn!("Bad password requested: {}", payload.email);
        return ServerResult::Error(CreateUserError::BadPassword);
    }

    let user_with_email = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db).await;

    match user_with_email {
        Ok(None) => (),
        Ok(_) => {
            warn!("Conflict email requested: {}", payload.email);
            return ServerResult::Error(CreateUserError::EmailExist);
        },
        Err(err) => {
            error!("{}", err);
            return ServerResult::Error(CreateUserError::InternalServerError);
        },
    }


    let transaction = &state.db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let uuid = Uuid::new_v4();
            user::ActiveModel {
                user_id: ActiveValue::Set(uuid),
                email: ActiveValue::Set(payload.email.clone()),
            }.insert(txn).await?;

            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hashed_password = argon2.hash_password(&payload.password.clone().into_bytes(), &salt).unwrap();

            password_identity::ActiveModel {
                user_id: ActiveValue::Set(uuid),
                password_hash: ActiveValue::Set(hashed_password.to_string()),
            }.insert(txn).await?;

            Ok(())
        })
    }).await;

    if let Err(err) = transaction {
        error!("{}", err);
        return ServerResult::Error(CreateUserError::InternalServerError);
    }


    ServerResult::Ok(CreateUserResponse{})
}
