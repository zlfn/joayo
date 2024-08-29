use api_derive::{FromImageError, FromSessionError};
use axum::{extract::State, http::StatusCode};
use axum_extra::{extract::{CookieJar, Multipart}, TypedHeader};
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use headers::ContentType;
use tracing::info;

use crate::{common::image::convert_to_avif, server_result::{ServerResult, ToStatusCode}};

#[derive(TryFromMultipart)]
pub struct CreateJoayoRequest {
    #[form_data(field_name = "image", limit = "50MiB")]
    image: Bytes,
    #[form_data(field_name = "image-type")]
    image_type: String,
}

#[derive(Deserialize)]
pub struct CreateJoayoRequestJson {
    test: String,
}

#[derive(Serialize, FromSessionError, FromImageError)]
pub enum CreateJoayoError {
    SessionInvalid,
    UnsupportedFormat,
    MissingMimeContentType,
    DecodeFailed,
    InternalServerError
}

impl ToStatusCode for CreateJoayoError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::UnsupportedFormat => return StatusCode::UNPROCESSABLE_ENTITY,
            Self::DecodeFailed => return StatusCode::UNPROCESSABLE_ENTITY,
            Self::MissingMimeContentType => return StatusCode::UNPROCESSABLE_ENTITY,
            Self::SessionInvalid => return StatusCode::UNAUTHORIZED,
            Self::InternalServerError => return StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn create_joayo(
    TypedHeader(content_type): TypedHeader<ContentType>,
    state: State<crate::AppState>,
    jar: CookieJar,
    multipart: TypedMultipart<CreateJoayoRequest>
) -> ServerResult<(), CreateJoayoError> {

    let avif = convert_to_avif::<CreateJoayoError>(&multipart.image, &multipart.image_type).await;

    if let Ok(avif) = avif {
        info!("{:?}", avif);
    }

    ServerResult::Ok(())
}
