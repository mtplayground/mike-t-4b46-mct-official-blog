use sqlx::{PgPool, Result};
use time::OffsetDateTime;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Media {
    pub id: i64,
    pub object_key: String,
    pub media_type: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MediaType {
    Image,
    Video,
}

impl MediaType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Video => "video",
        }
    }
}

pub struct CreateMediaInput {
    pub object_key: String,
    pub media_type: MediaType,
    pub content_type: String,
    pub size_bytes: i64,
}

pub async fn create_media(pool: &PgPool, input: CreateMediaInput) -> Result<Media> {
    sqlx::query_as::<_, Media>(
        r#"
        INSERT INTO media (object_key, media_type, content_type, size_bytes)
        VALUES ($1, $2, $3, $4)
        RETURNING id, object_key, media_type, content_type, size_bytes, created_at
        "#,
    )
    .bind(input.object_key)
    .bind(input.media_type.as_str())
    .bind(input.content_type)
    .bind(input.size_bytes)
    .fetch_one(pool)
    .await
}

pub async fn get_media_by_id(pool: &PgPool, id: i64) -> Result<Option<Media>> {
    sqlx::query_as::<_, Media>(
        r#"
        SELECT id, object_key, media_type, content_type, size_bytes, created_at
        FROM media
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_media_by_object_key(pool: &PgPool, object_key: &str) -> Result<Option<Media>> {
    sqlx::query_as::<_, Media>(
        r#"
        SELECT id, object_key, media_type, content_type, size_bytes, created_at
        FROM media
        WHERE object_key = $1
        "#,
    )
    .bind(object_key)
    .fetch_optional(pool)
    .await
}

pub async fn list_media(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Media>> {
    sqlx::query_as::<_, Media>(
        r#"
        SELECT id, object_key, media_type, content_type, size_bytes, created_at
        FROM media
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn list_media_by_type(
    pool: &PgPool,
    media_type: MediaType,
    limit: i64,
    offset: i64,
) -> Result<Vec<Media>> {
    sqlx::query_as::<_, Media>(
        r#"
        SELECT id, object_key, media_type, content_type, size_bytes, created_at
        FROM media
        WHERE media_type = $1
        ORDER BY created_at DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(media_type.as_str())
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn delete_media(pool: &PgPool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM media WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
