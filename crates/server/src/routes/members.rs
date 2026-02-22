use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

/// GET /servers/{server_id}/members — list all members of a server
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<rusteze_db::members::MemberWithUser>>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let members = rusteze_db::members::fetch_server_members(&state.db, server_id).await?;
    Ok(Json(members))
}

/// DELETE /servers/{server_id}/members/{user_id} — kick a member (owner only)
pub async fn kick_member(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, target_user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify caller is server owner
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user.0).await?;
    let is_owner = servers.iter().any(|s| s.id == server_id && s.owner_id == user.0);
    if !is_owner {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "only the server owner can kick members".into(),
        });
    }

    if target_user_id == user.0 {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "cannot kick yourself".into(),
        });
    }

    rusteze_db::members::remove_member(&state.db, server_id, target_user_id).await?;

    // Notify the kicked user via their personal channel
    let event = rusteze_models::ServerEvent::MemberLeave {
        server_id,
        user_id: target_user_id,
    };
    if let Ok(payload) = serde_json::to_string(&event) {
        let _: Result<(), _> = fred::interfaces::PubsubInterface::publish(
            &state.redis,
            format!("user:{target_user_id}"),
            payload.as_str(),
        )
        .await;
    }

    Ok(Json(serde_json::json!({"kicked": true})))
}

/// DELETE /servers/{server_id}/members/@me — leave a server
pub async fn leave_server(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Check if user is the owner
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user.0).await?;
    let is_owner = servers.iter().any(|s| s.id == server_id && s.owner_id == user.0);
    if is_owner {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "server owner cannot leave. transfer ownership or delete the server".into(),
        });
    }

    rusteze_db::members::remove_member(&state.db, server_id, user.0).await?;
    Ok(Json(serde_json::json!({"left": true})))
}
