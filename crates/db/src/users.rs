use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password_hash: String,
    pub flags: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> DbResult<UserRow> {
    let id = Uuid::now_v7();
    let disc = format!("{:04}", rand::random::<u16>() % 10000);

    let row: UserRow = sqlx::query_as(
        "INSERT INTO users (id, username, discriminator, email, password_hash) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(id)
    .bind(username)
    .bind(disc)
    .bind(email)
    .bind(password_hash)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> DbResult<UserRow> {
    let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    row.ok_or(crate::DbError::NotFound)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> DbResult<UserRow> {
    let row: Option<UserRow> = sqlx::query_as("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?;

    row.ok_or(crate::DbError::NotFound)
}
