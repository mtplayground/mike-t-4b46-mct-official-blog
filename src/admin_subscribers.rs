use std::{error::Error, fmt};

use axum::{
    Extension, Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::subscribers;

#[derive(Serialize)]
pub struct SubscriberResponse {
    pub id: i64,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug)]
pub enum AdminSubscribersError {
    ListDatabase(sqlx::Error),
}

pub async fn list_subscribers(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<SubscriberResponse>>, AdminSubscribersError> {
    let subscribers = subscribers::list_subscribers(&pool, 500, 0)
        .await
        .map_err(AdminSubscribersError::ListDatabase)?;

    Ok(Json(
        subscribers
            .into_iter()
            .map(|subscriber| SubscriberResponse {
                id: subscriber.id,
                email: subscriber.email,
                created_at: subscriber.created_at.to_string(),
            })
            .collect(),
    ))
}

impl IntoResponse for AdminSubscribersError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        eprintln!("admin subscribers request failed: {self}");

        (status, Json(ErrorResponse { error: self.message() })).into_response()
    }
}

impl AdminSubscribersError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ListDatabase(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::ListDatabase(_) => "Could not load subscribers.".to_owned(),
        }
    }
}

impl fmt::Display for AdminSubscribersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ListDatabase(error) => write!(f, "failed to list subscribers: {error}"),
        }
    }
}

impl Error for AdminSubscribersError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ListDatabase(error) => Some(error),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
