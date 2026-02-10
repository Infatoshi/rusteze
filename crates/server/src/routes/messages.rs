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

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<MessageQuery>,
) -> Result<Json<Vec<rusteze_db::messages::MessageRow>>, ApiError> {
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
