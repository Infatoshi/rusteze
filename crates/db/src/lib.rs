use sqlx::PgPool;
use thiserror::Error;

pub mod attachments;
pub mod channels;
pub mod dms;
pub mod invites;
pub mod members;
pub mod messages;
pub mod oauth;
pub mod reactions;
pub mod read_states;
pub mod roles;
pub mod servers;
pub mod users;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("not found")]
    NotFound,
    #[error("already exists")]
    AlreadyExists,
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub type DbResult<T> = Result<T, DbError>;

/// Create a connection pool from a database URL.
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;
    tracing::info!("connected to PostgreSQL");
    Ok(pool)
}

/// Run all pending migrations.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("migrations applied");
    Ok(())
}
