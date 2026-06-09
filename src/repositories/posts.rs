use sqlx::{PgPool, Result};
use time::OffsetDateTime;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Category {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub category_id: i64,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub published_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PostStatus {
    Draft,
    Published,
}

impl PostStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Published => "published",
        }
    }
}

pub struct CreatePostInput {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub category_id: i64,
    pub status: PostStatus,
}

pub struct UpdatePostInput {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub category_id: i64,
    pub status: PostStatus,
}

pub async fn list_categories(pool: &PgPool) -> Result<Vec<Category>> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, slug, name, description, created_at, updated_at
        FROM categories
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_category_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Category>> {
    sqlx::query_as::<_, Category>(
        r#"
        SELECT id, slug, name, description, created_at, updated_at
        FROM categories
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn create_post(pool: &PgPool, input: CreatePostInput) -> Result<Post> {
    let status = input.status.as_str();

    sqlx::query_as::<_, Post>(
        r#"
        INSERT INTO posts (title, slug, body, category_id, status, published_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $5 = 'published' THEN NOW() ELSE NULL END)
        RETURNING id, category_id, title, slug, body, status, published_at, created_at, updated_at
        "#,
    )
    .bind(input.title)
    .bind(input.slug)
    .bind(input.body)
    .bind(input.category_id)
    .bind(status)
    .fetch_one(pool)
    .await
}

pub async fn update_post(pool: &PgPool, id: i64, input: UpdatePostInput) -> Result<Option<Post>> {
    let status = input.status.as_str();

    sqlx::query_as::<_, Post>(
        r#"
        UPDATE posts
        SET
            title = $2,
            slug = $3,
            body = $4,
            category_id = $5,
            status = $6,
            published_at = CASE
                WHEN $6 = 'published' THEN COALESCE(published_at, NOW())
                ELSE NULL
            END
        WHERE id = $1
        RETURNING id, category_id, title, slug, body, status, published_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(input.title)
    .bind(input.slug)
    .bind(input.body)
    .bind(input.category_id)
    .bind(status)
    .fetch_optional(pool)
    .await
}

pub async fn set_post_status(
    pool: &PgPool,
    id: i64,
    status: PostStatus,
) -> Result<Option<Post>> {
    let status = status.as_str();

    sqlx::query_as::<_, Post>(
        r#"
        UPDATE posts
        SET
            status = $2,
            published_at = CASE
                WHEN $2 = 'published' THEN COALESCE(published_at, NOW())
                ELSE NULL
            END
        WHERE id = $1
        RETURNING id, category_id, title, slug, body, status, published_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_optional(pool)
    .await
}

pub async fn delete_post(pool: &PgPool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM posts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn get_post_by_id(pool: &PgPool, id: i64) -> Result<Option<Post>> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT id, category_id, title, slug, body, status, published_at, created_at, updated_at
        FROM posts
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_post_by_slug(pool: &PgPool, slug: &str) -> Result<Option<Post>> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT id, category_id, title, slug, body, status, published_at, created_at, updated_at
        FROM posts
        WHERE slug = $1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
}

pub async fn list_posts(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Post>> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT id, category_id, title, slug, body, status, published_at, created_at, updated_at
        FROM posts
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn list_published_posts(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Post>> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT id, category_id, title, slug, body, status, published_at, created_at, updated_at
        FROM posts
        WHERE status = 'published'
        ORDER BY published_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_published_posts(pool: &PgPool) -> Result<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM posts
        WHERE status = 'published'
        "#,
    )
    .fetch_one(pool)
    .await
}

pub async fn count_published_posts_by_category(
    pool: &PgPool,
    category_slug: &str,
) -> Result<i64> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM posts
        INNER JOIN categories ON categories.id = posts.category_id
        WHERE posts.status = 'published'
          AND categories.slug = $1
        "#,
    )
    .bind(category_slug)
    .fetch_one(pool)
    .await
}

pub async fn list_published_posts_by_category(
    pool: &PgPool,
    category_slug: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Post>> {
    sqlx::query_as::<_, Post>(
        r#"
        SELECT posts.id, posts.category_id, posts.title, posts.slug, posts.body, posts.status,
               posts.published_at, posts.created_at, posts.updated_at
        FROM posts
        INNER JOIN categories ON categories.id = posts.category_id
        WHERE posts.status = 'published'
          AND categories.slug = $1
        ORDER BY posts.published_at DESC, posts.id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(category_slug)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
