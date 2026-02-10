use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct ServerRow {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_server(pool: &PgPool, name: &str, owner_id: Uuid) -> DbResult<ServerRow> {
    let id = Uuid::now_v7();

    let row: ServerRow = sqlx::query_as(
        "INSERT INTO servers (id, name, owner_id) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(id)
    .bind(name)
    .bind(owner_id)
    .fetch_one(pool)
    .await?;

    // Add owner as member
    sqlx::query("INSERT INTO members (server_id, user_id) VALUES ($1, $2)")
        .bind(id)
        .bind(owner_id)
        .execute(pool)
        .await?;

    // Auto-create #general text channel
    let channel_id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO channels (id, server_id, name, channel_type) VALUES ($1, $2, 'general', 'text')",
    )
    .bind(channel_id)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_user_servers(pool: &PgPool, user_id: Uuid) -> DbResult<Vec<ServerRow>> {
    let rows: Vec<ServerRow> = sqlx::query_as(
        "SELECT s.* FROM servers s INNER JOIN members m ON m.server_id = s.id WHERE m.user_id = $1 ORDER BY s.created_at",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
