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

pub async fn update_server(
    pool: &PgPool,
    id: Uuid,
    name: Option<&str>,
    description: Option<Option<&str>>,
) -> DbResult<ServerRow> {
    let current: ServerRow = sqlx::query_as("SELECT * FROM servers WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(crate::DbError::NotFound)?;

    let new_name = name.unwrap_or(&current.name);
    let new_desc = match description {
        Some(d) => d.map(|s| s.to_string()),
        None => current.description,
    };

    let row: ServerRow = sqlx::query_as(
        "UPDATE servers SET name = $1, description = $2 WHERE id = $3 RETURNING *",
    )
    .bind(new_name)
    .bind(new_desc)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete_server(pool: &PgPool, id: Uuid) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM servers WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
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
