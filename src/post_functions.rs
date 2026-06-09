use leptos::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AdminCategoryRecord {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AdminPostRecord {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[server(prefix = "/admin/api")]
pub async fn list_admin_categories() -> Result<Vec<AdminCategoryRecord>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts;

        let pool = database_pool()?;
        let categories = posts::list_categories(&pool)
            .await
            .map_err(|error| database_error("list categories", error))?;

        Ok(categories.into_iter().map(category_to_record).collect())
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(server_only_error("list categories"))
    }
}

#[server(prefix = "/admin/api")]
pub async fn list_admin_posts(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<AdminPostRecord>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts;

        let pool = database_pool()?;
        let limit = clamp_limit(limit.unwrap_or(50));
        let offset = normalize_offset(offset.unwrap_or(0));
        let posts = posts::list_posts(&pool, limit, offset)
            .await
            .map_err(|error| database_error("list posts", error))?;

        Ok(posts.into_iter().map(post_to_record).collect())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (limit, offset);
        Err(server_only_error("list posts"))
    }
}

#[server(prefix = "/admin/api")]
pub async fn get_admin_post(id: i64) -> Result<Option<AdminPostRecord>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts;

        let pool = database_pool()?;
        let id = validate_id(id, "post id")?;
        let post = posts::get_post_by_id(&pool, id)
            .await
            .map_err(|error| database_error("load post", error))?;

        Ok(post.map(post_to_record))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        Err(server_only_error("load post"))
    }
}

#[server(prefix = "/admin/api")]
pub async fn create_admin_post(
    title: String,
    slug: String,
    body: String,
    category_id: i64,
    status: String,
) -> Result<AdminPostRecord, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts::{self, CreatePostInput};

        let pool = database_pool()?;
        let input = CreatePostInput {
            title: validate_title(title)?,
            slug: validate_slug(slug)?,
            body,
            category_id: validate_id(category_id, "category id")?,
            status: parse_status(&status)?,
        };
        let post = posts::create_post(&pool, input)
            .await
            .map_err(|error| database_error("create post", error))?;

        Ok(post_to_record(post))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (title, slug, body, category_id, status);
        Err(server_only_error("create post"))
    }
}

#[server(prefix = "/admin/api")]
pub async fn update_admin_post(
    id: i64,
    title: String,
    slug: String,
    body: String,
    category_id: i64,
    status: String,
) -> Result<AdminPostRecord, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts::{self, UpdatePostInput};

        let pool = database_pool()?;
        let id = validate_id(id, "post id")?;
        let input = UpdatePostInput {
            title: validate_title(title)?,
            slug: validate_slug(slug)?,
            body,
            category_id: validate_id(category_id, "category id")?,
            status: parse_status(&status)?,
        };
        let post = posts::update_post(&pool, id, input)
            .await
            .map_err(|error| database_error("update post", error))?
            .ok_or_else(|| not_found_error("Post"))?;

        Ok(post_to_record(post))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, title, slug, body, category_id, status);
        Err(server_only_error("update post"))
    }
}

#[server(prefix = "/admin/api")]
pub async fn delete_admin_post(id: i64) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts;

        let pool = database_pool()?;
        let id = validate_id(id, "post id")?;

        posts::delete_post(&pool, id)
            .await
            .map_err(|error| database_error("delete post", error))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = id;
        Err(server_only_error("delete post"))
    }
}

#[server(prefix = "/admin/api")]
pub async fn set_admin_post_published(
    id: i64,
    published: bool,
) -> Result<AdminPostRecord, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::repositories::posts::{self, PostStatus};

        let pool = database_pool()?;
        let id = validate_id(id, "post id")?;
        let status = if published {
            PostStatus::Published
        } else {
            PostStatus::Draft
        };
        let post = posts::set_post_status(&pool, id, status)
            .await
            .map_err(|error| database_error("change post status", error))?
            .ok_or_else(|| not_found_error("Post"))?;

        Ok(post_to_record(post))
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (id, published);
        Err(server_only_error("change post status"))
    }
}

fn validate_title(title: String) -> Result<String, ServerFnError> {
    let title = title.trim().to_owned();
    if title.is_empty() {
        return Err(validation_error("Title is required."));
    }
    if title.chars().count() > 220 {
        return Err(validation_error("Title must be 220 characters or fewer."));
    }

    Ok(title)
}

fn validate_slug(slug: String) -> Result<String, ServerFnError> {
    let slug = slug.trim().to_ascii_lowercase();
    if slug.is_empty() {
        return Err(validation_error("Slug is required."));
    }
    if slug.len() > 180 {
        return Err(validation_error("Slug must be 180 bytes or fewer."));
    }
    if !slug_has_valid_shape(&slug) {
        return Err(validation_error(
            "Slug must use lowercase letters, numbers, and single hyphens.",
        ));
    }

    Ok(slug)
}

fn slug_has_valid_shape(slug: &str) -> bool {
    let mut previous_hyphen = false;
    let mut saw_character = false;

    for byte in slug.bytes() {
        let is_alphanumeric = byte.is_ascii_lowercase() || byte.is_ascii_digit();
        if is_alphanumeric {
            previous_hyphen = false;
            saw_character = true;
        } else if byte == b'-' && saw_character && !previous_hyphen {
            previous_hyphen = true;
        } else {
            return false;
        }
    }

    saw_character && !previous_hyphen
}

fn validate_id(id: i64, name: &str) -> Result<i64, ServerFnError> {
    if id > 0 {
        Ok(id)
    } else {
        Err(validation_error(&format!("{name} must be a positive integer.")))
    }
}

fn clamp_limit(limit: i64) -> i64 {
    limit.clamp(1, 100)
}

fn normalize_offset(offset: i64) -> i64 {
    offset.max(0)
}

fn validation_error(message: &str) -> ServerFnError {
    ServerFnError::ServerError(message.to_owned())
}

fn not_found_error(resource: &str) -> ServerFnError {
    ServerFnError::ServerError(format!("{resource} was not found."))
}

fn server_only_error(action: &str) -> ServerFnError {
    ServerFnError::ServerError(format!("{action} is only available on the server."))
}

#[cfg(feature = "ssr")]
fn database_pool() -> Result<sqlx::PgPool, ServerFnError> {
    use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::ServerError("Database pool is unavailable.".to_owned()))
}

#[cfg(feature = "ssr")]
fn parse_status(status: &str) -> Result<crate::repositories::posts::PostStatus, ServerFnError> {
    match status.trim().to_ascii_lowercase().as_str() {
        "draft" => Ok(crate::repositories::posts::PostStatus::Draft),
        "published" => Ok(crate::repositories::posts::PostStatus::Published),
        _ => Err(validation_error("Status must be draft or published.")),
    }
}

#[cfg(feature = "ssr")]
fn database_error(action: &str, error: sqlx::Error) -> ServerFnError {
    eprintln!("failed to {action}: {error}");
    ServerFnError::ServerError(format!("Could not {action}."))
}

#[cfg(feature = "ssr")]
fn category_to_record(category: crate::repositories::posts::Category) -> AdminCategoryRecord {
    AdminCategoryRecord {
        id: category.id,
        slug: category.slug,
        name: category.name,
        description: category.description,
    }
}

#[cfg(feature = "ssr")]
fn post_to_record(post: crate::repositories::posts::Post) -> AdminPostRecord {
    AdminPostRecord {
        id: post.id,
        category_id: post.category_id,
        title: post.title,
        slug: post.slug,
        body: post.body,
        status: post.status,
        published_at: post.published_at.map(|value| value.to_string()),
        created_at: post.created_at.to_string(),
        updated_at: post.updated_at.to_string(),
    }
}
