use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct MemberRow {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub nickname: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

pub async fn is_member(pool: &PgPool, server_id: Uuid, user_id: Uuid) -> DbResult<bool> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM members WHERE server_id = $1 AND user_id = $2)",
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn add_member(pool: &PgPool, server_id: Uuid, user_id: Uuid) -> DbResult<MemberRow> {
    let row: MemberRow = sqlx::query_as(
        "INSERT INTO members (server_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING RETURNING *",
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(crate::DbError::AlreadyExists)?;

    Ok(row)
}

/// Get all channel IDs a user has access to (via their server memberships).
pub async fn user_channel_ids(pool: &PgPool, user_id: Uuid) -> DbResult<Vec<Uuid>> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT c.id FROM channels c INNER JOIN members m ON m.server_id = c.server_id WHERE m.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

/// Fetch all members of a server (with user info).
#[derive(Debug, serde::Serialize, FromRow)]
pub struct MemberWithUser {
    pub server_id: Uuid,
    pub user_id: Uuid,
    pub nickname: Option<String>,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn fetch_server_members(pool: &PgPool, server_id: Uuid) -> DbResult<Vec<MemberWithUser>> {
    let rows: Vec<MemberWithUser> = sqlx::query_as(
        r#"
        SELECT m.server_id, m.user_id, m.nickname, m.joined_at,
               u.username, u.discriminator, u.display_name, u.avatar_url
        FROM members m
        INNER JOIN users u ON u.id = m.user_id
        WHERE m.server_id = $1
        ORDER BY m.joined_at
        "#,
    )
    .bind(server_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn remove_member(pool: &PgPool, server_id: Uuid, user_id: Uuid) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM members WHERE server_id = $1 AND user_id = $2")
        .bind(server_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
}

/// Get the server_id for a given channel.
pub async fn channel_server_id(pool: &PgPool, channel_id: Uuid) -> DbResult<Option<Uuid>> {
    let row: Option<(Option<Uuid>,)> =
        sqlx::query_as("SELECT server_id FROM channels WHERE id = $1")
            .bind(channel_id)
            .fetch_optional(pool)
            .await?;

    match row {
        Some((server_id,)) => Ok(server_id),
        None => Err(crate::DbError::NotFound),
    }
}
