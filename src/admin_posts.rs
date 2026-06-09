use std::{collections::HashMap, error::Error, fmt};

use axum::{
    Extension, Json,
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::repositories::posts::{self, CreatePostInput, PostStatus, UpdatePostInput};

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
pub struct CategoryResponse {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Serialize)]
pub struct PostDetail {
    pub id: i64,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub category_id: i64,
    pub published_at: Option<String>,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct PostFormInput {
    pub title: String,
    pub category_id: i64,
    pub body: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct DeletePostResponse {
    pub deleted: bool,
}

#[derive(Debug)]
pub enum AdminPostsError {
    InvalidPostId,
    InvalidInput(String),
    NotFound,
    ListDatabase(sqlx::Error),
    LoadDatabase(sqlx::Error),
    WriteDatabase(sqlx::Error),
    DeleteDatabase(sqlx::Error),
}

pub async fn list_categories(
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Vec<CategoryResponse>>, AdminPostsError> {
    let categories = posts::list_categories(&pool)
        .await
        .map_err(AdminPostsError::ListDatabase)?;

    Ok(Json(
        categories
            .into_iter()
            .map(|category| CategoryResponse {
                id: category.id,
                name: category.name,
                slug: category.slug,
            })
            .collect(),
    ))
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

pub async fn get_post(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<PostDetail>, AdminPostsError> {
    let id = validate_post_id(id)?;
    let post = posts::get_post_by_id(&pool, id)
        .await
        .map_err(AdminPostsError::LoadDatabase)?
        .ok_or(AdminPostsError::NotFound)?;

    Ok(Json(post_to_detail(post)))
}

pub async fn create_post(
    Extension(pool): Extension<PgPool>,
    Json(input): Json<PostFormInput>,
) -> Result<Json<PostDetail>, AdminPostsError> {
    let input = normalize_post_input(input)?;
    let post = posts::create_post(
        &pool,
        CreatePostInput {
            title: input.title,
            slug: input.slug,
            body: input.body,
            category_id: input.category_id,
            status: input.status,
        },
    )
    .await
    .map_err(AdminPostsError::WriteDatabase)?;

    Ok(Json(post_to_detail(post)))
}

pub async fn update_post(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i64>,
    Json(input): Json<PostFormInput>,
) -> Result<Json<PostDetail>, AdminPostsError> {
    let id = validate_post_id(id)?;
    let input = normalize_post_input(input)?;
    let post = posts::update_post(
        &pool,
        id,
        UpdatePostInput {
            title: input.title,
            slug: input.slug,
            body: input.body,
            category_id: input.category_id,
            status: input.status,
        },
    )
    .await
    .map_err(AdminPostsError::WriteDatabase)?
    .ok_or(AdminPostsError::NotFound)?;

    Ok(Json(post_to_detail(post)))
}

pub async fn delete_post(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<DeletePostResponse>, AdminPostsError> {
    let id = validate_post_id(id)?;

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
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::ListDatabase(_)
            | Self::LoadDatabase(_)
            | Self::WriteDatabase(_)
            | Self::DeleteDatabase(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::InvalidPostId => "Post id must be a positive integer.".to_owned(),
            Self::InvalidInput(message) => message.clone(),
            Self::NotFound => "Post was not found.".to_owned(),
            Self::ListDatabase(_) => "Could not load posts.".to_owned(),
            Self::LoadDatabase(_) => "Could not load post.".to_owned(),
            Self::WriteDatabase(_) => "Could not save post.".to_owned(),
            Self::DeleteDatabase(_) => "Could not delete post.".to_owned(),
        }
    }
}

impl fmt::Display for AdminPostsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPostId => write!(f, "invalid post id"),
            Self::InvalidInput(message) => write!(f, "invalid post input: {message}"),
            Self::NotFound => write!(f, "post was not found"),
            Self::ListDatabase(error) => write!(f, "failed to list posts: {error}"),
            Self::LoadDatabase(error) => write!(f, "failed to load post: {error}"),
            Self::WriteDatabase(error) => write!(f, "failed to save post: {error}"),
            Self::DeleteDatabase(error) => write!(f, "failed to delete post: {error}"),
        }
    }
}

impl Error for AdminPostsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ListDatabase(error)
            | Self::LoadDatabase(error)
            | Self::WriteDatabase(error)
            | Self::DeleteDatabase(error) => Some(error),
            Self::InvalidPostId | Self::InvalidInput(_) | Self::NotFound => None,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

struct NormalizedPostInput {
    title: String,
    slug: String,
    body: String,
    category_id: i64,
    status: PostStatus,
}

fn normalize_post_input(input: PostFormInput) -> Result<NormalizedPostInput, AdminPostsError> {
    let title = input.title.trim().to_owned();
    if title.is_empty() {
        return Err(AdminPostsError::InvalidInput("Title is required.".to_owned()));
    }
    if title.chars().count() > 220 {
        return Err(AdminPostsError::InvalidInput(
            "Title must be 220 characters or fewer.".to_owned(),
        ));
    }
    if input.category_id <= 0 {
        return Err(AdminPostsError::InvalidInput(
            "Category is required.".to_owned(),
        ));
    }
    if input.body.len() > 500_000 {
        return Err(AdminPostsError::InvalidInput(
            "Body must be 500000 bytes or fewer.".to_owned(),
        ));
    }

    Ok(NormalizedPostInput {
        slug: slugify(&title),
        title,
        body: input.body,
        category_id: input.category_id,
        status: parse_status(&input.status)?,
    })
}

fn parse_status(status: &str) -> Result<PostStatus, AdminPostsError> {
    match status.trim().to_ascii_lowercase().as_str() {
        "draft" => Ok(PostStatus::Draft),
        "published" => Ok(PostStatus::Published),
        _ => Err(AdminPostsError::InvalidInput(
            "Status must be draft or published.".to_owned(),
        )),
    }
}

fn validate_post_id(id: i64) -> Result<i64, AdminPostsError> {
    if id > 0 {
        Ok(id)
    } else {
        Err(AdminPostsError::InvalidPostId)
    }
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut needs_hyphen = false;

    for byte in title.bytes() {
        let lower = byte.to_ascii_lowercase();
        if lower.is_ascii_lowercase() || lower.is_ascii_digit() {
            if needs_hyphen && !slug.is_empty() {
                slug.push('-');
            }
            slug.push(char::from(lower));
            needs_hyphen = false;
        } else if !slug.is_empty() {
            needs_hyphen = true;
        }

        if slug.len() >= 180 {
            break;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "post".to_owned()
    } else {
        slug
    }
}

fn post_to_detail(post: posts::Post) -> PostDetail {
    PostDetail {
        id: post.id,
        title: post.title,
        slug: post.slug,
        body: post.body,
        status: post.status,
        category_id: post.category_id,
        published_at: post.published_at.map(|value| value.to_string()),
        updated_at: post.updated_at.to_string(),
    }
}
