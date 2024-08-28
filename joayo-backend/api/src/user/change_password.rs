use api_derive::FromSessionError;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use chrono::{TimeDelta, Utc};
use rand::rngs::OsRng;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DbErr, EntityTrait, QueryFilter, TransactionTrait};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{common::session::get_user_from_session, server_result::{Json, ServerResult, ToStatusCode}};
use crate::entities::{prelude::*, *};


#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    old_password: String,
    new_password: String,
}

#[derive(Serialize, FromSessionError)]
pub enum ChangePasswordError {
    BadNewPassword,
    WrongOldPassword,
    Unauthorized,
    SessionInvalid,
    InternalServerError
}

impl ToStatusCode for ChangePasswordError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::BadNewPassword => return StatusCode::UNPROCESSABLE_ENTITY,
            Self::WrongOldPassword => return StatusCode::UNAUTHORIZED,
            Self::Unauthorized => return StatusCode::UNAUTHORIZED,
            Self::SessionInvalid => return StatusCode::FORBIDDEN,
            Self::InternalServerError => return StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn change_password(
    state: State<crate::AppState>,
    jar: CookieJar,
    Json(payload): Json<ChangePasswordRequest>,
) -> (CookieJar, ServerResult<(), ChangePasswordError>) {

    let user = match get_user_from_session::<ChangePasswordError>(&jar, &state.db).await {
        Ok(user) => user,
        Err(err) => return (jar, ServerResult::Error(err))
    };

    if payload.new_password.as_bytes().len() < 8 || payload.new_password.as_bytes().len() > 128 {
        warn!("Bad new password requested: {}", user.email);
        return (jar, ServerResult::Error(ChangePasswordError::BadNewPassword));
    }

    let password_identity = PasswordIdentity::find_by_id(user.user_id)
        .one(&state.db)
        .await;

    let password_identity = match password_identity {
        Ok(None) => {
            error!("Password identity not found for user-id {}", user.user_id);
            return (jar, ServerResult::Error(ChangePasswordError::InternalServerError));
        },
        Err(err) => {
            error!("{}", err);
            return (jar, ServerResult::Error(ChangePasswordError::InternalServerError));
        },
        Ok(Some(password_identity)) => password_identity
    };

    let old_password = payload.old_password.as_bytes();

    let parsed_hash = PasswordHash::new(&password_identity.password_hash);

    let parsed_hash = match parsed_hash {
        Ok(parsed_hash) => parsed_hash,
        Err(err) => {
            error!("{}", err);
            return (jar, ServerResult::Error(ChangePasswordError::InternalServerError));
        }
    };

    if Argon2::default().verify_password(old_password, &parsed_hash).is_err() {
        warn!("Wrong old password requested: {}", user.email);
        return (jar, ServerResult::Error(ChangePasswordError::WrongOldPassword));
    }

    let session_id = Uuid::new_v4();
    let transaction = &state.db.transaction::<_, _, DbErr>(|txn| {
        Box::pin(async move {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let hashed_password = argon2.hash_password(&payload.new_password.clone().into_bytes(), &salt).unwrap();

            password_identity::ActiveModel {
                user_id: ActiveValue::Set(user.user_id),
                password_hash: ActiveValue::Set(hashed_password.to_string()),
            }.update(txn).await?;

            Session::delete_many()
                .filter(session::Column::UserId.eq(user.user_id))
                .exec(txn)
                .await?;

            session::ActiveModel {
                user_id: ActiveValue::set(user.user_id),
                session_id: ActiveValue::set(session_id.clone()),
                created_at: ActiveValue::set(Utc::now().naive_utc()),
                expires_at: ActiveValue::set(Utc::now().naive_utc() + TimeDelta::minutes(30)),
            }.insert(txn).await?;

            Ok(())
        })
    }).await;

    if let Err(err) = transaction {
        error!("{}", err);
        return (jar, ServerResult::Error(ChangePasswordError::InternalServerError));
    }

    let jar = jar.add(Cookie::new("session_id", session_id.to_string()));

    (jar, ServerResult::Ok(()))
}
