use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct ReadStateRow {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub last_read_id: Option<Uuid>,
    pub mention_count: i32,
}

/// Update the last-read message for a user in a channel.
pub async fn update_read_state(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
    last_read_id: Uuid,
) -> DbResult<()> {
    sqlx::query(
        r#"
        INSERT INTO read_states (user_id, channel_id, last_read_id, mention_count)
        VALUES ($1, $2, $3, 0)
        ON CONFLICT (user_id, channel_id) DO UPDATE
        SET last_read_id = $3, mention_count = 0
        "#,
    )
    .bind(user_id)
    .bind(channel_id)
    .bind(last_read_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get all read states for a user (for computing unread indicators).
pub async fn get_user_read_states(
    pool: &PgPool,
    user_id: Uuid,
) -> DbResult<Vec<ReadStateRow>> {
    let rows: Vec<ReadStateRow> = sqlx::query_as(
        "SELECT * FROM read_states WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Increment the mention count for a user in a channel.
pub async fn increment_mentions(
    pool: &PgPool,
    user_id: Uuid,
    channel_id: Uuid,
) -> DbResult<()> {
    sqlx::query(
        r#"
        INSERT INTO read_states (user_id, channel_id, mention_count)
        VALUES ($1, $2, 1)
        ON CONFLICT (user_id, channel_id) DO UPDATE
        SET mention_count = read_states.mention_count + 1
        "#,
    )
    .bind(user_id)
    .bind(channel_id)
    .execute(pool)
    .await?;

    Ok(())
}
