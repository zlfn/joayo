use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;


#[derive(Serialize, Clone)]
#[serde(tag="result", content = "data")]
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

        (status, Json(self)).into_response()
    }
}
