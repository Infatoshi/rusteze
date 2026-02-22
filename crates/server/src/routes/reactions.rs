use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct ReactionRequest {
    pub emoji: String,
}

/// PUT /channels/{channel_id}/messages/{message_id}/reactions
pub async fn add_reaction(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<ReactionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify channel access
    verify_channel_access(&state, user.0, channel_id).await?;

    rusteze_db::reactions::add_reaction(&state.db, message_id, user.0, &body.emoji).await?;

    // Broadcast reaction event
    let event = rusteze_models::ServerEvent::ReactionAdd {
        channel_id,
        message_id,
        user_id: user.0,
        emoji: body.emoji,
    };

    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("channel:{channel_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

/// DELETE /channels/{channel_id}/messages/{message_id}/reactions/{emoji}
pub async fn remove_reaction(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((channel_id, message_id, emoji)): Path<(Uuid, Uuid, String)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;

    rusteze_db::reactions::remove_reaction(&state.db, message_id, user.0, &emoji).await?;

    let event = rusteze_models::ServerEvent::ReactionRemove {
        channel_id,
        message_id,
        user_id: user.0,
        emoji,
    };

    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("channel:{channel_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(serde_json::json!({"ok": true})))
}

/// GET /channels/{channel_id}/messages/{message_id}/reactions
pub async fn list_reactions(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((channel_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<rusteze_db::reactions::ReactionCount>>, ApiError> {
    verify_channel_access(&state, user.0, channel_id).await?;
    let reactions = rusteze_db::reactions::fetch_reactions(&state.db, message_id).await?;
    Ok(Json(reactions))
}

async fn verify_channel_access(
    state: &AppState,
    user_id: Uuid,
    channel_id: Uuid,
) -> Result<(), ApiError> {
    let server_id = rusteze_db::members::channel_server_id(&state.db, channel_id).await?;

    match server_id {
        Some(sid) => {
            if !rusteze_db::members::is_member(&state.db, sid, user_id).await? {
                return Err(ApiError {
                    status: axum::http::StatusCode::FORBIDDEN,
                    message: "not a member of this server".into(),
                });
            }
        }
        None => {
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
