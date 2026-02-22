use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct CreateDmRequest {
    pub recipient_id: Uuid,
}

#[derive(Deserialize)]
pub struct CreateGroupDmRequest {
    pub name: String,
    pub member_ids: Vec<Uuid>,
}

/// POST /dms — open or get a 1-on-1 DM channel
pub async fn create_dm(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<CreateDmRequest>,
) -> Result<Json<rusteze_db::channels::ChannelRow>, ApiError> {
    if body.recipient_id == user.0 {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "cannot DM yourself".into(),
        });
    }

    // Verify recipient exists
    rusteze_db::users::find_by_id(&state.db, body.recipient_id).await?;

    let channel = rusteze_db::dms::get_or_create_dm(&state.db, user.0, body.recipient_id).await?;

    // Subscribe both users to this channel via Redis
    let event = rusteze_models::ServerEvent::ChannelCreate(rusteze_models::Channel {
        id: channel.id,
        server_id: channel.server_id,
        name: channel.name.clone(),
        channel_type: rusteze_models::ChannelType::DirectMessage,
        topic: channel.topic.clone(),
        position: channel.position,
        created_at: channel.created_at,
    });

    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("user:{}", user.0),
            payload.as_str(),
        )
        .await;
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("user:{}", body.recipient_id),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(channel))
}

/// POST /dms/group — create a group DM
pub async fn create_group_dm(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<CreateGroupDmRequest>,
) -> Result<Json<rusteze_db::channels::ChannelRow>, ApiError> {
    if body.member_ids.is_empty() {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "group DM must have at least one other member".into(),
        });
    }

    if body.member_ids.len() > 9 {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "group DM cannot exceed 10 members".into(),
        });
    }

    let channel =
        rusteze_db::dms::create_group_dm(&state.db, user.0, &body.name, &body.member_ids).await?;
    Ok(Json(channel))
}

/// GET /dms — list all DM channels for the authenticated user
pub async fn list_dms(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<rusteze_db::dms::DmChannelInfo>>, ApiError> {
    let dms = rusteze_db::dms::fetch_user_dms(&state.db, user.0).await?;
    Ok(Json(dms))
}
