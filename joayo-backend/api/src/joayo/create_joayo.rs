use api_derive::FromSessionError;
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use serde::Serialize;
use tracing::error;

use crate::{common::session::get_user_from_session, common::result::{ServerResult, ToStatusCode}};

#[derive(TryFromMultipart)]
pub struct CreateJoayoRequest {
    #[form_data(field_name = "image", limit = "50MB")]
    image: Bytes,
}

#[derive(Serialize, FromSessionError)]
pub enum CreateJoayoError {
    SessionInvalid,
    EncodingRequestFailed,
    InternalServerError
}

impl ToStatusCode for CreateJoayoError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::SessionInvalid => return StatusCode::UNAUTHORIZED,
            Self::EncodingRequestFailed => return StatusCode::SERVICE_UNAVAILABLE,
            Self::InternalServerError => return StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn create_joayo(
    state: State<crate::AppState>,
    jar: CookieJar,
    multipart: TypedMultipart<CreateJoayoRequest>
) -> ServerResult<(), CreateJoayoError> {

    let _user = match get_user_from_session::<CreateJoayoError>(&jar, &state.db).await {
        Ok(user) => user,
        Err(err) => return ServerResult::Error(err)
    };

    //TODO: Register JOAYO to database.

    let image_send = state.image_tx.send(multipart.image.clone());
    if let Err(err) = image_send {
        error!("Failed to send image to queue: {}", err);
        return ServerResult::Error(CreateJoayoError::EncodingRequestFailed);
    }

    ServerResult::Ok(())
}
