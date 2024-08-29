use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use log::error;
use sea_orm::EntityTrait;
use serde::Serialize;
use tracing::warn;
use uuid::Uuid;

use crate::server_result::{ServerResult, ToStatusCode};
use crate::entities::prelude::*;


#[derive(Serialize)]
pub enum DeleteSessionError {
    SessionInvalid,
    InternalServerError
}

impl ToStatusCode for DeleteSessionError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::SessionInvalid => StatusCode::UNAUTHORIZED,
            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn delete_session(
    state: State<crate::AppState>,
    jar: CookieJar
) -> (CookieJar, ServerResult<(), DeleteSessionError>){

    let session_id = match jar.get("session_id") {
        Some(session_id) => match Uuid::parse_str(&session_id.value()) {
            Ok(session_id) => session_id,
            Err(_) => {
                warn!("Invalid session_id requested: {}", session_id.value());
                return (jar, ServerResult::Error(DeleteSessionError::SessionInvalid))
            }
        }
        None => return (jar, ServerResult::Error(DeleteSessionError::SessionInvalid))
    };

    let delete_result = Session::delete_by_id(session_id)
        .exec(&state.db)
        .await;

    match delete_result {
        Ok(_) => {
            let jar = jar.remove(Cookie::from("session_id"));
            (jar, ServerResult::Ok(()))
        }
        Err(err) => {
            error!("{}", err);
            (jar, ServerResult::Error(DeleteSessionError::InternalServerError))
        },
    }
}
