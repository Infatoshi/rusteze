use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct ChannelRow {
    pub id: Uuid,
    pub server_id: Option<Uuid>,
    pub name: String,
    pub channel_type: String,
    pub topic: Option<String>,
    pub position: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_channel(
    pool: &PgPool,
    server_id: Uuid,
    name: &str,
    channel_type: &str,
) -> DbResult<ChannelRow> {
    let id = Uuid::now_v7();

    let row: ChannelRow = sqlx::query_as(
        "INSERT INTO channels (id, server_id, name, channel_type) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(id)
    .bind(Some(server_id))
    .bind(name)
    .bind(channel_type)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_channel(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    topic: Option<Option<&str>>,
) -> DbResult<ChannelRow> {
    let current: ChannelRow = sqlx::query_as("SELECT * FROM channels WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(crate::DbError::NotFound)?;

    let new_name = name.unwrap_or(&current.name);
    let new_topic = match topic {
        Some(t) => t.map(|s| s.to_string()),
        None => current.topic,
    };

    let row: ChannelRow = sqlx::query_as(
        "UPDATE channels SET name = $1, topic = $2 WHERE id = $3 RETURNING *",
    )
    .bind(new_name)
    .bind(new_topic)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete_channel(pool: &PgPool, id: Uuid) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM channels WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
}

pub async fn fetch_server_channels(pool: &PgPool, server_id: Uuid) -> DbResult<Vec<ChannelRow>> {
    let rows: Vec<ChannelRow> =
        sqlx::query_as("SELECT * FROM channels WHERE server_id = $1 ORDER BY position")
            .bind(Some(server_id))
            .fetch_all(pool)
            .await?;

    Ok(rows)
}
