use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct InviteRow {
    pub code: String,
    pub server_id: Uuid,
    pub channel_id: Option<Uuid>,
    pub creator_id: Uuid,
    pub max_uses: Option<i32>,
    pub uses: i32,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create_invite(
    pool: &PgPool,
    server_id: Uuid,
    creator_id: Uuid,
    code: &str,
) -> DbResult<InviteRow> {
    let row: InviteRow = sqlx::query_as(
        "INSERT INTO invites (code, server_id, creator_id) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(code)
    .bind(server_id)
    .bind(creator_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn use_invite(pool: &PgPool, code: &str) -> DbResult<InviteRow> {
    let row: Option<InviteRow> = sqlx::query_as(
        "UPDATE invites SET uses = uses + 1 WHERE code = $1 AND (max_uses IS NULL OR uses < max_uses) AND (expires_at IS NULL OR expires_at > now()) RETURNING *",
    )
    .bind(code)
    .fetch_optional(pool)
    .await?;

    row.ok_or(crate::DbError::NotFound)
}
