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

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

#[derive(Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

/// Check that the user is a member of the server that owns this channel.
async fn verify_channel_access(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<(), ApiError> {
    // Check if it's a DM channel first
    let server_id = rusteze_db::members::channel_server_id(&state.db, channel_id).await?;

    match server_id {
        Some(sid) => {
            // Server channel — verify server membership
            if !rusteze_db::members::is_member(&state.db, sid, user_id).await? {
                return Err(ApiError {
                    status: axum::http::StatusCode::FORBIDDEN,
                    message: "not a member of this server".into(),
                });
            }
        }
        None => {
            // DM channel — verify DM membership
            if !rusteze_db::dms::is_dm_member(&state.db, channel_id, user_id).await? {
                return Err(ApiError {
                    status: axum::http::StatusCode::FORBIDDEN,
                    message: "not a participant of this conversation".into(),
                });
            }
        }
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

pub async fn edit_message(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<EditMessageRequest>,
) -> Result<Json<rusteze_db::messages::MessageRow>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;

    let msg = rusteze_db::messages::update_message(
        &state.db,
        message_id,
        channel_id,
        user.0,
        &body.content,
    )
    .await?;

    // Publish update event
    let event = rusteze_models::ServerEvent::MessageUpdate {
        id: msg.id,
        channel_id: msg.channel_id,
        content: msg.content.clone(),
        edited_at: msg.edited_at,
    };

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

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;

    // Any member with channel access can delete messages (server owner/admin model TBD)
    rusteze_db::messages::delete_message(&state.db, message_id, channel_id).await?;

    // Publish delete event
    let event = rusteze_models::ServerEvent::MessageDelete {
        id: message_id,
        channel_id,
    };

    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("channel:{channel_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(serde_json::json!({"deleted": true})))
}

pub async fn pin_message(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<rusteze_db::messages::MessageRow>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;
    let msg = rusteze_db::messages::pin_message(&state.db, message_id, channel_id).await?;
    Ok(Json(msg))
}

pub async fn list_pins(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<rusteze_db::messages::MessageRow>>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;
    let pins = rusteze_db::messages::fetch_pinned(&state.db, channel_id).await?;
    Ok(Json(pins))
}

pub async fn search_messages(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<rusteze_db::messages::MessageRow>>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;
    let limit = query.limit.unwrap_or(25).min(100);
    let results =
        rusteze_db::messages::search_messages(&state.db, channel_id, &query.q, limit).await?;
    Ok(Json(results))
}
