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

pub async fn fetch_server_channels(pool: &PgPool, server_id: Uuid) -> DbResult<Vec<ChannelRow>> {
    let rows: Vec<ChannelRow> =
        sqlx::query_as("SELECT * FROM channels WHERE server_id = $1 ORDER BY position")
            .bind(Some(server_id))
            .fetch_all(pool)
            .await?;

    Ok(rows)
}
