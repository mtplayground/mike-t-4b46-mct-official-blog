use std::{collections::HashMap, error::Error, fmt};

use axum::{
    Extension, Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::posts;

const RECENT_POST_LIMIT: i64 = 6;
const EXCERPT_CHAR_LIMIT: usize = 160;

#[derive(Debug, Serialize)]
pub struct PublicPostCard {
    pub title: String,
    pub slug: String,
    pub category: String,
    pub excerpt: String,
    pub published_at: Option<i64>,
}

#[derive(Debug, Serialize)]
struct ErrorMessage {
    error: &'static str,
}

#[derive(Debug)]
pub enum PublicPostsError {
    Database(sqlx::Error),
}

impl fmt::Display for PublicPostsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(_) => write!(f, "failed to load public posts"),
        }
    }
}

impl Error for PublicPostsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
        }
    }
}

impl IntoResponse for PublicPostsError {
    fn into_response(self) -> Response {
        eprintln!("public posts error: {self:?}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                error: "Could not load recent posts. Please try again.",
            }),
        )
            .into_response()
    }
}

pub async fn recent_posts(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<PublicPostCard>>, PublicPostsError> {
    let categories = posts::list_categories(&pool)
        .await
        .map_err(PublicPostsError::Database)?;
    let category_names = categories
        .into_iter()
        .map(|category| (category.id, category.name))
        .collect::<HashMap<_, _>>();

    let recent_posts = posts::list_published_posts(&pool, RECENT_POST_LIMIT, 0)
        .await
        .map_err(PublicPostsError::Database)?;

    let cards = recent_posts
        .into_iter()
        .map(|post| PublicPostCard {
            title: post.title,
            slug: post.slug,
            category: category_names
                .get(&post.category_id)
                .cloned()
                .unwrap_or_else(|| "Post".to_owned()),
            excerpt: excerpt_from_body(&post.body),
            published_at: post.published_at.map(|published_at| published_at.unix_timestamp()),
        })
        .collect();

    Ok(Json(cards))
}

fn excerpt_from_body(body: &str) -> String {
    let normalized = body.split_whitespace().collect::<Vec<_>>().join(" ");

    if normalized.chars().count() <= EXCERPT_CHAR_LIMIT {
        return normalized;
    }

    let mut excerpt = normalized
        .chars()
        .take(EXCERPT_CHAR_LIMIT)
        .collect::<String>();
    excerpt.push_str("...");
    excerpt
}
