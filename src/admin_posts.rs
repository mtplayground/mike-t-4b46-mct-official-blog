use std::{collections::HashMap, error::Error, fmt};

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::posts;

#[derive(Serialize)]
pub struct DashboardPost {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub category_id: i64,
    pub category_name: String,
    pub published_at: Option<String>,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct DeletePostResponse {
    pub deleted: bool,
}

#[derive(Debug)]
pub enum AdminPostsError {
    InvalidPostId,
    ListDatabase(sqlx::Error),
    DeleteDatabase(sqlx::Error),
}

pub async fn list_posts(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<DashboardPost>>, AdminPostsError> {
    let categories = posts::list_categories(&pool)
        .await
        .map_err(AdminPostsError::ListDatabase)?;
    let category_names: HashMap<i64, String> = categories
        .into_iter()
        .map(|category| (category.id, category.name))
        .collect();
    let posts = posts::list_posts(&pool, 100, 0)
        .await
        .map_err(AdminPostsError::ListDatabase)?;

    Ok(Json(
        posts
            .into_iter()
            .map(|post| {
                let category_name = category_names
                    .get(&post.category_id)
                    .cloned()
                    .unwrap_or_else(|| "Unknown category".to_owned());

                DashboardPost {
                    id: post.id,
                    title: post.title,
                    slug: post.slug,
                    status: post.status,
                    category_id: post.category_id,
                    category_name,
                    published_at: post.published_at.map(|value| value.to_string()),
                    updated_at: post.updated_at.to_string(),
                }
            })
            .collect(),
    ))
}

pub async fn delete_post(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<DeletePostResponse>, AdminPostsError> {
    if id <= 0 {
        return Err(AdminPostsError::InvalidPostId);
    }

    let deleted = posts::delete_post(&pool, id)
        .await
        .map_err(AdminPostsError::DeleteDatabase)?;

    Ok(Json(DeletePostResponse { deleted }))
}

impl IntoResponse for AdminPostsError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        eprintln!("admin posts request failed: {self}");

        (status, Json(ErrorResponse { error: self.message() })).into_response()
    }
}

impl AdminPostsError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidPostId => StatusCode::BAD_REQUEST,
            Self::ListDatabase(_) | Self::DeleteDatabase(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidPostId => "Post id must be a positive integer.".to_owned(),
            Self::ListDatabase(_) => "Could not load posts.".to_owned(),
            Self::DeleteDatabase(_) => "Could not delete post.".to_owned(),
        }
    }
}

impl fmt::Display for AdminPostsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPostId => write!(f, "invalid post id"),
            Self::ListDatabase(error) => write!(f, "failed to list posts: {error}"),
            Self::DeleteDatabase(error) => write!(f, "failed to delete post: {error}"),
        }
    }
}

impl Error for AdminPostsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ListDatabase(error) | Self::DeleteDatabase(error) => Some(error),
            Self::InvalidPostId => None,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}
