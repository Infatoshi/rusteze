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

pub async fn create_channel(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
    Json(body): Json<CreateChannelRequest>,
) -> Result<Json<rusteze_db::channels::ChannelRow>, ApiError> {
    // Verify user is a member
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let channel =
        rusteze_db::channels::create_channel(&state.db, server_id, &body.name, &body.channel_type)
            .await?;
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
