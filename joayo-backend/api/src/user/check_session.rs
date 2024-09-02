use api_derive::{FromSessionError, ToStatusCode};
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use serde::Serialize;

use crate::common::session::get_user_from_session;
use crate::common::result::{ServerResult, ToStatusCode};

#[derive(Serialize, ToStatusCode)]
#[status_code(OK)]
pub struct CheckSessionResponse {
    email: String,
}

#[derive(Serialize, FromSessionError)]
pub enum CheckSessionError {
    SessionInvalid,
    InternalServerError
}

impl ToStatusCode for CheckSessionError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::SessionInvalid => return StatusCode::UNAUTHORIZED,
            Self::InternalServerError => return StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn check_session(
    state: State<crate::AppState>,
    jar: CookieJar
) -> ServerResult<CheckSessionResponse, CheckSessionError>{

    let user = match get_user_from_session::<CheckSessionError>(&jar, &state.db).await {
        Ok(user) => user,
        Err(err) => return ServerResult::Error(err)
    };

    ServerResult::Ok(CheckSessionResponse {
        email: user.email
    })
}
