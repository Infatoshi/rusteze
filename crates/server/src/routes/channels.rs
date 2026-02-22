use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default = "default_channel_type")]
    pub channel_type: String,
}

fn default_channel_type() -> String {
    "text".into()
}

#[derive(Deserialize)]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
}

pub async fn create_channel(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
    Json(body): Json<CreateChannelRequest>,
) -> Result<Json<rusteze_db::channels::ChannelRow>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let name = body.name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "channel name must be 1-100 characters".into(),
        });
    }

    let channel =
        rusteze_db::channels::create_channel(&state.db, server_id, name, &body.channel_type)
            .await?;

    // Broadcast ChannelCreate to all members via Redis
    let event = rusteze_models::ServerEvent::ChannelCreate(rusteze_models::Channel {
        id: channel.id,
        server_id: channel.server_id,
        name: channel.name.clone(),
        channel_type: match channel.channel_type.as_str() {
            "voice" => rusteze_models::ChannelType::Voice,
            _ => rusteze_models::ChannelType::Text,
        },
        topic: channel.topic.clone(),
        position: channel.position,
        created_at: channel.created_at,
    });
    if let Ok(payload) = serde_json::to_string(&event) {
        // Broadcast to the first channel in this server (all members are subscribed to #general)
        // This avoids sending N duplicate events for N subscribed channels
        let channels = rusteze_db::channels::fetch_server_channels(&state.db, server_id)
            .await
            .unwrap_or_default();
        if let Some(first_ch) = channels.first() {
            let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
                &state.redis,
                format!("channel:{}", first_ch.id),
                payload.as_str(),
            )
            .await;
        }
    }

    Ok(Json(channel))
}

pub async fn list_channels(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<rusteze_db::channels::ChannelRow>>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let channels = rusteze_db::channels::fetch_server_channels(&state.db, server_id).await?;
    Ok(Json(channels))
}

pub async fn update_channel(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateChannelRequest>,
) -> Result<Json<rusteze_db::channels::ChannelRow>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let channel = rusteze_db::channels::update_channel(
        &state.db,
        channel_id,
        body.name.as_deref(),
        body.topic.as_ref().map(|t| Some(t.as_str())),
    )
    .await?;

    // Broadcast ChannelUpdate
    let event = rusteze_models::ServerEvent::ChannelUpdate {
        id: channel.id,
        name: Some(channel.name.clone()),
        topic: channel.topic.clone(),
    };
    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("channel:{channel_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(channel))
}

pub async fn delete_channel(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    rusteze_db::channels::delete_channel(&state.db, channel_id).await?;

    // Broadcast ChannelDelete to first remaining channel
    let event = rusteze_models::ServerEvent::ChannelDelete { id: channel_id };
    if let Ok(payload) = serde_json::to_string(&event) {
        let channels = rusteze_db::channels::fetch_server_channels(&state.db, server_id)
            .await
            .unwrap_or_default();
        if let Some(first_ch) = channels.first() {
            let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
                &state.redis,
                format!("channel:{}", first_ch.id),
                payload.as_str(),
            )
            .await;
        }
    }

    Ok(Json(serde_json::json!({"deleted": true})))
}
