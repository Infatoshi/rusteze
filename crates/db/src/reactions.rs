use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct ReactionRow {
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Aggregated reaction count for a message.
#[derive(Debug, serde::Serialize, FromRow)]
pub struct ReactionCount {
    pub emoji: String,
    pub count: i64,
}

pub async fn add_reaction(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> DbResult<()> {
    sqlx::query(
        "INSERT INTO reactions (message_id, user_id, emoji) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(message_id)
    .bind(user_id)
    .bind(emoji)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn remove_reaction(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> DbResult<()> {
    let result = sqlx::query(
        "DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3",
    )
    .bind(message_id)
    .bind(user_id)
    .bind(emoji)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
}

pub async fn fetch_reactions(pool: &PgPool, message_id: Uuid) -> DbResult<Vec<ReactionCount>> {
    let rows: Vec<ReactionCount> = sqlx::query_as(
        "SELECT emoji, COUNT(*) as count FROM reactions WHERE message_id = $1 GROUP BY emoji ORDER BY MIN(created_at)",
    )
    .bind(message_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Get which emojis a specific user has reacted with on a message.
pub async fn user_reactions(
    pool: &PgPool,
    message_id: Uuid,
    user_id: Uuid,
) -> DbResult<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT emoji FROM reactions WHERE message_id = $1 AND user_id = $2",
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(e,)| e).collect())
}
