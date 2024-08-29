use axum_extra::extract::CookieJar;
use chrono::{TimeDelta, Utc};
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tracing::{error, warn};
use uuid::Uuid;

use crate::entities::{prelude::*, *};

pub enum SessionError {
    SessionInvalid,
    InternalServerError,
}

pub trait FromSessionError {
    fn from_session_error(session_error: SessionError) -> Self;
}

pub async fn get_user_from_session<T: FromSessionError>
(jar: &CookieJar, db: &DatabaseConnection) -> Result<user::Model, T> {

    let session_id = match jar.get("session_id") {
        Some(session_id) => match Uuid::parse_str(&session_id.value()) {
            Ok(session_id) => session_id,
            Err(_) => {
                warn!("Invalid session_id requested: {}", session_id.value());
                return Result::Err(T::from_session_error(SessionError::SessionInvalid))
            }
        }
        None => return Result::Err(T::from_session_error(SessionError::SessionInvalid))
    };

    let session = Session::find_by_id(session_id)
        .one(db)
        .await;

    let session = match session {
        Ok(Some(session)) => session,
        Ok(None) => return Result::Err(T::from_session_error(SessionError::SessionInvalid)),
        Err(err) => {
            error!("{}", err);
            return Result::Err(T::from_session_error(SessionError::InternalServerError));
        }
    };

    if session.expires_at < Utc::now().naive_utc() {
        return Result::Err(T::from_session_error(SessionError::SessionInvalid))
    }

    let mut session_update: session::ActiveModel = session.clone().into();
    session_update.expires_at = Set(Utc::now().naive_utc() + TimeDelta::minutes(30));

    if let Err(err) = session_update.update(db).await {
        error!("{}", err);
        return Result::Err(T::from_session_error(SessionError::InternalServerError))
    }

    let user = User::find_by_id(session.user_id)
        .one(db)
        .await;

    let user = match user {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("Session is valid but user not found: {}, {}", session.user_id, session.session_id);
            return Result::Err(T::from_session_error(SessionError::InternalServerError))
        }
        Err(err) => {
            error!("{}", err);
            return Result::Err(T::from_session_error(SessionError::InternalServerError))
        }
    };

    Result::Ok(user)
}
