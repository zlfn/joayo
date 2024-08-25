use axum::{extract::rejection::JsonRejection, http::StatusCode, response::IntoResponse};
use axum_macros::FromRequest;
use serde::Serialize;
use tracing::warn;

#[derive(Serialize, Clone)]
#[serde(tag="type", content="data")]
pub enum ServerResult<T, E> 
where 
    T: Serialize,
    E: Serialize,
{
    Ok(T),
    Error(E),
}

pub trait ToStatusCode {
    fn to_status_code(&self) -> StatusCode;
}

impl ToStatusCode for () {
    fn to_status_code(&self) -> StatusCode {
        StatusCode::OK
    }
}

impl<T, E> IntoResponse for ServerResult<T, E> 
where  
    T: Serialize + ToStatusCode,
    E: Serialize + ToStatusCode, 
{
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            ServerResult::Ok(data) => data.to_status_code(),
            ServerResult::Error(err) => err.to_status_code()
        };

        (status, axum::Json(self)).into_response()
    }
}

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(JsonError))]
pub struct Json<T>(pub T);

#[derive(Serialize, Debug)]
pub enum JsonError {
    JsonDataError,
    JsonSyntaxError,
    JsonContentTypeError,
    JsonBytesError,
    JsonUnknownError,
}

impl From<JsonRejection> for JsonError {
    fn from(rejection: JsonRejection) -> Self {
        warn!("{}: {}", rejection.status(), rejection.body_text());
        match rejection {
            JsonRejection::JsonSyntaxError(_) => JsonError::JsonSyntaxError,
            JsonRejection::JsonDataError(_) => JsonError::JsonDataError,
            JsonRejection::MissingJsonContentType(_) => JsonError::JsonContentTypeError,
            JsonRejection::BytesRejection(_) => JsonError::JsonBytesError,
            _ => JsonError::JsonUnknownError,
        }
    }
}

impl ToStatusCode for JsonError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            Self::JsonSyntaxError => StatusCode::BAD_REQUEST,
            Self::JsonDataError => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JsonContentTypeError => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::JsonBytesError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JsonUnknownError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> axum::response::Response {
        ServerResult::<(), JsonError>::Error(self).into_response()
    }
}
