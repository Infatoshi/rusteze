use sqlx::PgPool;
use thiserror::Error;

pub mod messages;
pub mod users;
pub mod servers;
pub mod channels;
pub mod members;
pub mod invites;

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
