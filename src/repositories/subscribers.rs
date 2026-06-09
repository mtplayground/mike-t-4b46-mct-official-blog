use sqlx::{PgPool, Result};
use time::OffsetDateTime;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Subscriber {
    pub id: i64,
    pub email: String,
    pub created_at: OffsetDateTime,
}

pub async fn create_subscriber(pool: &PgPool, email: &str) -> Result<Subscriber> {
    let inserted = sqlx::query_as::<_, Subscriber>(
        r#"
        INSERT INTO newsletter_subscribers (email)
        VALUES ($1)
        ON CONFLICT DO NOTHING
        RETURNING id, email, created_at
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    match inserted {
        Some(subscriber) => Ok(subscriber),
        None => get_subscriber_by_email(pool, email)
            .await?
            .ok_or(sqlx::Error::RowNotFound),
    }
}

pub async fn get_subscriber_by_email(pool: &PgPool, email: &str) -> Result<Option<Subscriber>> {
    sqlx::query_as::<_, Subscriber>(
        r#"
        SELECT id, email, created_at
        FROM newsletter_subscribers
        WHERE lower(email) = lower($1)
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn list_subscribers(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Subscriber>> {
    sqlx::query_as::<_, Subscriber>(
        r#"
        SELECT id, email, created_at
        FROM newsletter_subscribers
        ORDER BY created_at DESC, id DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn delete_subscriber(pool: &PgPool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM newsletter_subscribers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
