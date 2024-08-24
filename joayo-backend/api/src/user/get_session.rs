use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, Json, http::StatusCode};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use chrono::{Utc, TimeDelta};
use sea_orm::prelude::*;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};
use uuid::Uuid;

use crate::server_result::{ToStatusCode, ServerResult};
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
pub struct GetSessionRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub enum GetSessionError {
    NoEmail,
    WrongPassword,
    InternalServerError,
}

impl ToStatusCode for GetSessionError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::NoEmail => StatusCode::FORBIDDEN,
            Self::WrongPassword => StatusCode::FORBIDDEN,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


pub async fn get_session(
    state: State<crate::AppState>,
    jar: CookieJar,
    Json(payload): Json<GetSessionRequest>
) -> (CookieJar, ServerResult<(), GetSessionError>) {

    let user = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db)
        .await;

    let user: user::Model = match user {
        Ok(None) => {
            warn!("Unregistered Email requested: {}", payload.email);
            return (jar, ServerResult::Error(GetSessionError::NoEmail));
        },
        Err(err) => {
            error!("{}", err);
            return (jar, ServerResult::Error(GetSessionError::InternalServerError));
        },
        Ok(Some(user)) => user,
    };

    let password_identity = PasswordIdentity::find_by_id(user.user_id)
        .one(&state.db)
        .await;

    let password_identity = match password_identity {
        Ok(None) => {
            error!("Password identity not found for user-id {}", user.user_id);
            return (jar, ServerResult::Error(GetSessionError::InternalServerError));
        },
        Err(err) => {
            error!("{}", err);
            return (jar, ServerResult::Error(GetSessionError::InternalServerError));
        },
        Ok(Some(password_identity)) => password_identity
    };

    let password = payload.password.as_bytes();

    let parsed_hash = PasswordHash::new(&password_identity.password_hash);

    let parsed_hash = match parsed_hash {
        Ok(parsed_hash) => parsed_hash,
        Err(err) => {
            error!("{}", err);
            return (jar, ServerResult::Error(GetSessionError::InternalServerError));
        }
    };

    if Argon2::default().verify_password(password, &parsed_hash).is_err() {
        warn!("Wrong password requested: {}", payload.email);
        return (jar, ServerResult::Error(GetSessionError::WrongPassword));
    }

    let session_id = Uuid::now_v7();

    let session = session::ActiveModel {
        user_id: ActiveValue::set(user.user_id),
        session_id: ActiveValue::set(session_id.clone()),
        created_at: ActiveValue::set(Utc::now().naive_utc()),
        expires_at: ActiveValue::set(Utc::now().naive_utc() + TimeDelta::minutes(30)),
    }.insert(&state.db).await;

    if let Err(err) = session {
        error!("{}", err);
        return (jar, ServerResult::Error(GetSessionError::InternalServerError));
    }

    let jar = jar.add(Cookie::new("session_id", session_id.to_string()));

    (jar, ServerResult::Ok(()))
}
