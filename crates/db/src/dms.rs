use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;
use crate::channels::ChannelRow;

/// A DM channel with the list of participant user IDs.
#[derive(Debug, serde::Serialize)]
pub struct DmChannelInfo {
    #[serde(flatten)]
    pub channel: ChannelRow,
    pub participants: Vec<Uuid>,
}

#[derive(Debug, serde::Serialize, FromRow)]
pub struct DmMemberRow {
    pub channel_id: Uuid,
    pub user_id: Uuid,
}

/// Create a 1-on-1 DM channel between two users. Returns existing channel if one already exists.
pub async fn get_or_create_dm(
    pool: &PgPool,
    user_a: Uuid,
    user_b: Uuid,
) -> DbResult<ChannelRow> {
    // Check if a DM between these two users already exists
    let existing: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT dm1.channel_id
        FROM dm_members dm1
        INNER JOIN dm_members dm2 ON dm1.channel_id = dm2.channel_id
        INNER JOIN channels c ON c.id = dm1.channel_id
        WHERE dm1.user_id = $1 AND dm2.user_id = $2
          AND c.channel_type = 'direct_message'
        LIMIT 1
        "#,
    )
    .bind(user_a)
    .bind(user_b)
    .fetch_optional(pool)
    .await?;

    if let Some((channel_id,)) = existing {
        let channel: ChannelRow = sqlx::query_as("SELECT * FROM channels WHERE id = $1")
            .bind(channel_id)
            .fetch_one(pool)
            .await?;
        return Ok(channel);
    }

    // Create new DM channel
    let channel_id = Uuid::now_v7();
    let channel: ChannelRow = sqlx::query_as(
        "INSERT INTO channels (id, server_id, name, channel_type) VALUES ($1, NULL, 'DM', 'direct_message') RETURNING *",
    )
    .bind(channel_id)
    .fetch_one(pool)
    .await?;

    // Add both users as DM members
    sqlx::query("INSERT INTO dm_members (channel_id, user_id) VALUES ($1, $2), ($1, $3)")
        .bind(channel_id)
        .bind(user_a)
        .bind(user_b)
        .execute(pool)
        .await?;

    Ok(channel)
}

/// Create a group DM channel with multiple users.
pub async fn create_group_dm(
    pool: &PgPool,
    creator_id: Uuid,
    name: &str,
    member_ids: &[Uuid],
) -> DbResult<ChannelRow> {
    let channel_id = Uuid::now_v7();
    let channel: ChannelRow = sqlx::query_as(
        "INSERT INTO channels (id, server_id, name, channel_type) VALUES ($1, NULL, $2, 'group_dm') RETURNING *",
    )
    .bind(channel_id)
    .bind(name)
    .fetch_one(pool)
    .await?;

    // Add creator
    sqlx::query("INSERT INTO dm_members (channel_id, user_id) VALUES ($1, $2)")
        .bind(channel_id)
        .bind(creator_id)
        .execute(pool)
        .await?;

    // Add other members
    for &member_id in member_ids {
        if member_id != creator_id {
            sqlx::query("INSERT INTO dm_members (channel_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(channel_id)
                .bind(member_id)
                .execute(pool)
                .await?;
        }
    }

    Ok(channel)
}

/// Fetch all DM channels for a user.
pub async fn fetch_user_dms(pool: &PgPool, user_id: Uuid) -> DbResult<Vec<DmChannelInfo>> {
    let channels: Vec<ChannelRow> = sqlx::query_as(
        r#"
        SELECT c.*
        FROM channels c
        INNER JOIN dm_members dm ON dm.channel_id = c.id
        WHERE dm.user_id = $1 AND c.server_id IS NULL
        ORDER BY c.created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::with_capacity(channels.len());
    for channel in channels {
        let participants: Vec<(Uuid,)> =
            sqlx::query_as("SELECT user_id FROM dm_members WHERE channel_id = $1")
                .bind(channel.id)
                .fetch_all(pool)
                .await?;

        result.push(DmChannelInfo {
            channel,
            participants: participants.into_iter().map(|(id,)| id).collect(),
        });
    }

    Ok(result)
}

/// Check if a user is a participant of a DM channel.
pub async fn is_dm_member(pool: &PgPool, channel_id: Uuid, user_id: Uuid) -> DbResult<bool> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM dm_members WHERE channel_id = $1 AND user_id = $2)",
    )
    .bind(channel_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
