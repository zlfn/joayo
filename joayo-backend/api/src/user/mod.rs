use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use crate::entities::{prelude::*, *};
use uuid::Uuid;


#[derive(Serialize)]
#[serde(tag="type", content = "data")]
pub enum ServerResult<T: Serialize, E: Serialize> {
    Ok(T),
    Error(E),
}

#[derive(Deserialize)]
pub struct CreateUser {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub enum CreateUserError {
    EmailExist,
    BadPassword,
    DatabaseError,
}

pub async fn create_user(
    state: State<crate::AppState>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<ServerResult<(), CreateUserError>>) {

    if payload.password.as_bytes().len() < 8 || payload.password.as_bytes().len() > 128 {
        return (StatusCode::BAD_REQUEST, Json(ServerResult::Error(CreateUserError::BadPassword)));
    }

    let user_with_email = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db).await;

    match user_with_email {
        Ok(None) => (),
        Ok(_) => return (StatusCode::CONFLICT, Json(ServerResult::Error(CreateUserError::EmailExist))),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ServerResult::Error(CreateUserError::DatabaseError))),
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
                salt: ActiveValue::Set(salt.to_string())
            }.insert(txn).await?;

            Ok(())
        })
    }).await;

    if let Err(_) = transaction {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ServerResult::Error(CreateUserError::DatabaseError)));
    }


    (StatusCode::CREATED, Json(ServerResult::Ok(())))
}
