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
    pub bio: Option<String>,
    pub banner_url: Option<String>,
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
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.code().as_deref() == Some("23505") {
                return crate::DbError::AlreadyExists;
            }
        }
        crate::DbError::Sqlx(e)
    })?;

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

pub async fn find_by_username(pool: &PgPool, username: &str) -> DbResult<Vec<UserRow>> {
    let rows: Vec<UserRow> = sqlx::query_as("SELECT * FROM users WHERE username = $1")
        .bind(username)
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

/// Update user profile fields. Only non-None values are updated.
pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    display_name: Option<Option<&str>>,
    avatar_url: Option<Option<&str>>,
    bio: Option<Option<&str>>,
    banner_url: Option<Option<&str>>,
) -> DbResult<UserRow> {
    let current = find_by_id(pool, user_id).await?;

    let new_display_name = match display_name {
        Some(v) => v.map(|s| s.to_string()),
        None => current.display_name,
    };
    let new_avatar = match avatar_url {
        Some(v) => v.map(|s| s.to_string()),
        None => current.avatar_url,
    };
    let new_bio = match bio {
        Some(v) => v.map(|s| s.to_string()),
        None => current.bio,
    };
    let new_banner = match banner_url {
        Some(v) => v.map(|s| s.to_string()),
        None => current.banner_url,
    };

    let row: UserRow = sqlx::query_as(
        "UPDATE users SET display_name = $1, avatar_url = $2, bio = $3, banner_url = $4 WHERE id = $5 RETURNING *",
    )
    .bind(new_display_name)
    .bind(new_avatar)
    .bind(new_bio)
    .bind(new_banner)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Create a user from OAuth (no password required).
pub async fn create_oauth_user(
    pool: &PgPool,
    username: &str,
    email: &str,
) -> DbResult<UserRow> {
    let id = Uuid::now_v7();
    let disc = format!("{:04}", rand::random::<u16>() % 10000);

    // OAuth users get an empty password_hash (they can't login with password)
    let row: UserRow = sqlx::query_as(
        "INSERT INTO users (id, username, discriminator, email, password_hash) VALUES ($1, $2, $3, $4, '') RETURNING *",
    )
    .bind(id)
    .bind(username)
    .bind(disc)
    .bind(email)
    .fetch_one(pool)
    .await?;

    Ok(row)
}
