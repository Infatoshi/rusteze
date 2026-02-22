use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<CreateServerRequest>,
) -> Result<Json<rusteze_db::servers::ServerRow>, ApiError> {
    let name = body.name.trim();
    if name.is_empty() || name.len() > 100 {
        return Err(ApiError {
            status: axum::http::StatusCode::BAD_REQUEST,
            message: "server name must be 1-100 characters".into(),
        });
    }
    let server = rusteze_db::servers::create_server(&state.db, name, user.0).await?;
    Ok(Json(server))
}

pub async fn list_servers(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<rusteze_db::servers::ServerRow>>, ApiError> {
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user.0).await?;
    Ok(Json(servers))
}

pub async fn update_server(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
    Json(body): Json<UpdateServerRequest>,
) -> Result<Json<rusteze_db::servers::ServerRow>, ApiError> {
    // Verify ownership
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user.0).await?;
    let is_owner = servers.iter().any(|s| s.id == server_id && s.owner_id == user.0);
    if !is_owner {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "only the server owner can update settings".into(),
        });
    }

    let server = rusteze_db::servers::update_server(
        &state.db,
        server_id,
        body.name.as_deref(),
        body.description.as_ref().map(|d| Some(d.as_str())),
    )
    .await?;

    Ok(Json(server))
}

pub async fn delete_server(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user.0).await?;
    let is_owner = servers.iter().any(|s| s.id == server_id && s.owner_id == user.0);
    if !is_owner {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "only the server owner can delete the server".into(),
        });
    }

    rusteze_db::servers::delete_server(&state.db, server_id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
