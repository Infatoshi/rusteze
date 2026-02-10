use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Deserialize;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
}

pub async fn create_server(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<CreateServerRequest>,
) -> Result<Json<rusteze_db::servers::ServerRow>, ApiError> {
    let server = rusteze_db::servers::create_server(&state.db, &body.name, user.0).await?;
    Ok(Json(server))
}

pub async fn list_servers(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<Vec<rusteze_db::servers::ServerRow>>, ApiError> {
    let servers = rusteze_db::servers::fetch_user_servers(&state.db, user.0).await?;
    Ok(Json(servers))
}
