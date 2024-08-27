use api_derive::ToStatusCode;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use chrono::{TimeDelta, Utc};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::Serialize;
use tracing::{error, warn};
use uuid::Uuid;

use crate::entities::{prelude::*, *};
use crate::server_result::{ServerResult, ToStatusCode};

#[derive(Serialize, ToStatusCode)]
#[status_code(OK)]
pub struct CheckSessionResponse {
    email: String,
}

#[derive(Serialize)]
pub enum CheckSessionError {
    Unauthorized,
    SessionExpired,
    InternalServerError
}

impl ToStatusCode for CheckSessionError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => return StatusCode::UNAUTHORIZED,
            Self::SessionExpired => return StatusCode::FORBIDDEN,
            Self::InternalServerError => return StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn check_session(
    state: State<crate::AppState>,
    jar: CookieJar
) -> ServerResult<CheckSessionResponse, CheckSessionError>{

    let session_id = match jar.get("session_id") {
        Some(session_id) => match Uuid::parse_str(&session_id.value()) {
            Ok(session_id) => session_id,
            Err(_) => {
                warn!("Invalid session_id requested: {}", session_id.value());
                return ServerResult::Error(CheckSessionError::Unauthorized)
            }
        }
        None => return ServerResult::Error(CheckSessionError::Unauthorized)
    };

    let session = Session::find_by_id(session_id)
        .one(&state.db)
        .await;

    let session = match session {
        Ok(Some(session)) => session,
        Ok(None) => {
            warn!("Session not found in database: {}", session_id);
            return ServerResult::Error(CheckSessionError::Unauthorized)
        },
        Err(err) => {
            error!("{}", err);
            return ServerResult::Error(CheckSessionError::InternalServerError)
        }
    };

    if session.expires_at < Utc::now().naive_utc() {
        return ServerResult::Error(CheckSessionError::SessionExpired)
    }

    let mut session_update: session::ActiveModel = session.clone().into();
    session_update.expires_at = Set(Utc::now().naive_utc() + TimeDelta::minutes(30));

    if let Err(err) = session_update.update(&state.db).await {
        error!("{}", err);
        return ServerResult::Error(CheckSessionError::InternalServerError)
    }

    let user = User::find_by_id(session.user_id)
        .one(&state.db)
        .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("Session is valid but user not found: {}, {}", session.user_id, session.session_id);
            return ServerResult::Error(CheckSessionError::InternalServerError)
        }
        Err(err) => {
            error!("{}", err);
            return ServerResult::Error(CheckSessionError::InternalServerError)
        }
    };

    ServerResult::Ok(CheckSessionResponse {
        email: user.email
    })
}
