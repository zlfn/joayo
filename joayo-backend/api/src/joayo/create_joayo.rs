use api_derive::FromSessionError;
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use axum_typed_multipart::{TryFromMultipart, TypedMultipart};
use bytes::Bytes;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serde::Serialize;
use service::image::ImageUploadRequest;
use tracing::error;
use uuid::Uuid;

use crate::{common::session::get_user_from_session, common::result::{ServerResult, ToStatusCode}};
use orm::entities::*;

#[derive(TryFromMultipart)]
pub struct CreateJoayoRequest {
    #[form_data(field_name = "image", limit = "35MB")]
    image: Bytes,
    #[form_data(field_name = "image_url")]
    image_url: Option<String>,
    #[form_data(field_name = "reference_url", limit = "50MB")]
    reference_url: Option<String>
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

    let user = match get_user_from_session::<CreateJoayoError>(&jar, &state.db).await {
        Ok(user) => user,
        Err(err) => return ServerResult::Error(err)
    };

    let joayo_id = Uuid::now_v7();
    let joayo_result = joayo::ActiveModel {
        user_id: ActiveValue::Set(user.user_id),
        joayo_id: ActiveValue::Set(joayo_id),
        created_at: ActiveValue::Set(Utc::now().naive_utc()),
        image_url: ActiveValue::Set(multipart.image_url.clone()),
        reference_url: ActiveValue::Set(multipart.reference_url.clone()),
    }.insert(&state.db).await;

    if let Err(err) = joayo_result {
        error!("Failed to insert JOAYO to DB: {}", err);
        return ServerResult::Error(CreateJoayoError::InternalServerError);
    }

    let image_send = state.image_tx.send(ImageUploadRequest {
        joayo_id,
        bytes: multipart.image.clone(),
        crf: 40
    });

    if let Err(err) = image_send {
        error!("Failed to send image to queue: {}", err);
        return ServerResult::Error(CreateJoayoError::EncodingRequestFailed);
    }

    ServerResult::Ok(())
}
