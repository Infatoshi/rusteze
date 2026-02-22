use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct AttachmentRow {
    pub id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_path: String,
    pub data: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Create an attachment with base64 data stored in the database.
pub async fn create_attachment_base64(
    pool: &PgPool,
    message_id: Uuid,
    filename: &str,
    content_type: &str,
    size: i64,
    base64_data: &str,
) -> DbResult<AttachmentRow> {
    let id = Uuid::now_v7();

    let row: AttachmentRow = sqlx::query_as(
        "INSERT INTO attachments (id, message_id, filename, content_type, size, storage_path, data) VALUES ($1, $2, $3, $4, $5, '', $6) RETURNING *",
    )
    .bind(id)
    .bind(message_id)
    .bind(filename)
    .bind(content_type)
    .bind(size)
    .bind(base64_data)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Create an attachment with filesystem storage (legacy).
pub async fn create_attachment(
    pool: &PgPool,
    message_id: Uuid,
    filename: &str,
    content_type: &str,
    size: i64,
    storage_path: &str,
) -> DbResult<AttachmentRow> {
    let id = Uuid::now_v7();

    let row: AttachmentRow = sqlx::query_as(
        "INSERT INTO attachments (id, message_id, filename, content_type, size, storage_path) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(id)
    .bind(message_id)
    .bind(filename)
    .bind(content_type)
    .bind(size)
    .bind(storage_path)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_message_attachments(
    pool: &PgPool,
    message_id: Uuid,
) -> DbResult<Vec<AttachmentRow>> {
    let rows: Vec<AttachmentRow> =
        sqlx::query_as("SELECT * FROM attachments WHERE message_id = $1 ORDER BY created_at")
            .bind(message_id)
            .fetch_all(pool)
            .await?;

    Ok(rows)
}

pub async fn fetch_attachment(pool: &PgPool, id: Uuid) -> DbResult<AttachmentRow> {
    let row: Option<AttachmentRow> = sqlx::query_as("SELECT * FROM attachments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    row.ok_or(crate::DbError::NotFound)
}
