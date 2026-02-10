use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub author_id: Uuid,
    pub content: Option<String>,
    pub replies_to: Option<Uuid>,
    pub pinned: bool,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_message(
    pool: &PgPool,
    channel_id: Uuid,
    author_id: Uuid,
    content: Option<&str>,
    replies_to: Option<Uuid>,
) -> DbResult<MessageRow> {
    let id = Uuid::now_v7();

    let row: MessageRow = sqlx::query_as(
        "INSERT INTO messages (id, channel_id, author_id, content, replies_to) VALUES ($1, $2, $3, $4, $5) RETURNING *",
    )
    .bind(id)
    .bind(channel_id)
    .bind(author_id)
    .bind(content)
    .bind(replies_to)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_messages(
    pool: &PgPool,
    channel_id: Uuid,
    before: Option<Uuid>,
    limit: i64,
) -> DbResult<Vec<MessageRow>> {
    let rows: Vec<MessageRow> = if let Some(before) = before {
        sqlx::query_as(
            "SELECT * FROM messages WHERE channel_id = $1 AND id < $2 ORDER BY id DESC LIMIT $3",
        )
        .bind(channel_id)
        .bind(before)
        .bind(limit)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as("SELECT * FROM messages WHERE channel_id = $1 ORDER BY id DESC LIMIT $2")
            .bind(channel_id)
            .bind(limit)
            .fetch_all(pool)
            .await?
    };

    Ok(rows)
}

pub async fn delete_message(pool: &PgPool, id: Uuid, channel_id: Uuid) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM messages WHERE id = $1 AND channel_id = $2")
        .bind(id)
        .bind(channel_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
}
