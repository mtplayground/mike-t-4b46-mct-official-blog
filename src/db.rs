use std::{error::Error, fmt, str::FromStr};

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use crate::config::DatabaseConfig;

const DEFAULT_MAX_CONNECTIONS: u32 = 5;

#[derive(Debug)]
pub enum DbError {
    InvalidDatabaseUrl(sqlx::Error),
    Connect(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
    Verify(sqlx::Error),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDatabaseUrl(_) => write!(f, "DATABASE_URL is not a valid Postgres URL"),
            Self::Connect(_) => write!(f, "failed to connect to Postgres"),
            Self::Migrate(_) => write!(f, "failed to run database migrations"),
            Self::Verify(_) => write!(f, "failed to verify Postgres connectivity"),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDatabaseUrl(error) => Some(error),
            Self::Connect(error) => Some(error),
            Self::Migrate(error) => Some(error),
            Self::Verify(error) => Some(error),
        }
    }
}

pub async fn connect(config: &DatabaseConfig) -> Result<PgPool, DbError> {
    connect_with_url(&config.url).await
}

pub async fn connect_with_url(database_url: &str) -> Result<PgPool, DbError> {
    let options = PgConnectOptions::from_str(database_url)
        .map_err(DbError::InvalidDatabaseUrl)?
        .statement_cache_capacity(0);

    PgPoolOptions::new()
        .max_connections(DEFAULT_MAX_CONNECTIONS)
        .connect_with(options)
        .await
        .map_err(DbError::Connect)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), DbError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(DbError::Migrate)
}

pub async fn verify_connectivity(pool: &PgPool) -> Result<(), DbError> {
    let _value: i32 = sqlx::query_scalar("SELECT 1")
        .persistent(false)
        .fetch_one(pool)
        .await
        .map_err(DbError::Verify)?;

    Ok(())
}
