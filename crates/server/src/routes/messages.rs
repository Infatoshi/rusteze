use std::sync::Arc;

use axum::{Json, extract::{Path, Query, State}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};
use rusteze_models::MessageCreate;

#[derive(Deserialize)]
pub struct MessageQuery {
    pub before: Option<Uuid>,
    pub limit: Option<i64>,
}

/// Check that the user is a member of the server that owns this channel.
async fn verify_channel_access(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<(), ApiError> {
    let server_id = rusteze_db::members::channel_server_id(&state.db, channel_id)
        .await?
        .ok_or(ApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            message: "channel not found".into(),
        })?;

    if !rusteze_db::members::is_member(&state.db, server_id, user_id).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }
    Ok(())
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Vec<rusteze_db::messages::MessageRow>>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;

    let limit = query.limit.unwrap_or(50).min(100);
    let messages =
        rusteze_db::messages::fetch_messages(&state.db, channel_id, query.before, limit).await?;
    Ok(Json(messages))
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Json(body): Json<MessageCreate>,
) -> Result<Json<rusteze_db::messages::MessageRow>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;

    let msg = rusteze_db::messages::create_message(
        &state.db,
        channel_id,
        user.0,
        body.content.as_deref(),
        body.replies_to,
    )
    .await?;

    // Publish event to Redis for gateway fan-out
    let event = rusteze_models::ServerEvent::MessageCreate(rusteze_models::Message {
        id: msg.id,
        channel_id: msg.channel_id,
        author_id: msg.author_id,
        content: msg.content.clone(),
        attachments: vec![],
        embeds: vec![],
        mentions: vec![],
        replies_to: msg.replies_to,
        pinned: msg.pinned,
        edited_at: msg.edited_at,
        created_at: msg.created_at,
    });

    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("channel:{channel_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(msg))
}
