use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub color: Option<i32>,
    pub permissions: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub color: Option<Option<i32>>,
    pub permissions: Option<i64>,
}

#[derive(Deserialize)]
pub struct RoleAssignment {
    pub user_id: Uuid,
}

/// Verify the user is the server owner (for admin operations).
async fn verify_owner(state: &AppState, server_id: Uuid, user_id: Uuid) -> Result<(), ApiError> {
    let server = rusteze_db::servers::fetch_user_servers(&state.db, user_id).await?;
    let is_owner = server.iter().any(|s| s.id == server_id && s.owner_id == user_id);
    if !is_owner {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "only the server owner can manage roles".into(),
        });
    }
    Ok(())
}

/// POST /servers/{server_id}/roles
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
    Json(body): Json<CreateRoleRequest>,
) -> Result<Json<rusteze_db::roles::RoleRow>, ApiError> {
    verify_owner(&state, server_id, user.0).await?;

    let role = rusteze_db::roles::create_role(
        &state.db,
        server_id,
        &body.name,
        body.color,
        body.permissions.unwrap_or(0),
    )
    .await?;

    Ok(Json(role))
}

/// GET /servers/{server_id}/roles
pub async fn list_roles(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
) -> Result<Json<Vec<rusteze_db::roles::RoleRow>>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let roles = rusteze_db::roles::fetch_server_roles(&state.db, server_id).await?;
    Ok(Json(roles))
}

/// PATCH /servers/{server_id}/roles/{role_id}
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, role_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateRoleRequest>,
) -> Result<Json<rusteze_db::roles::RoleRow>, ApiError> {
    verify_owner(&state, server_id, user.0).await?;

    let role = rusteze_db::roles::update_role(
        &state.db,
        role_id,
        body.name.as_deref(),
        body.color,
        body.permissions,
    )
    .await?;

    Ok(Json(role))
}

/// DELETE /servers/{server_id}/roles/{role_id}
pub async fn delete_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_owner(&state, server_id, user.0).await?;
    rusteze_db::roles::delete_role(&state.db, role_id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

/// PUT /servers/{server_id}/roles/{role_id}/members
pub async fn assign_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, role_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<RoleAssignment>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_owner(&state, server_id, user.0).await?;

    // Verify target user is a member
    if !rusteze_db::members::is_member(&state.db, server_id, body.user_id).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "user is not a member of this server".into(),
        });
    }

    rusteze_db::roles::assign_role(&state.db, server_id, body.user_id, role_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

/// DELETE /servers/{server_id}/roles/{role_id}/members/{user_id}
pub async fn revoke_role(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path((server_id, role_id, target_user_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    verify_owner(&state, server_id, user.0).await?;
    rusteze_db::roles::revoke_role(&state.db, server_id, target_user_id, role_id).await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
