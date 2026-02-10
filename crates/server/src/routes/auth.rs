use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, state::AppState};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub user_id: uuid::Uuid,
    pub token: String,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let result = rusteze_auth::session::register(
        &state.db,
        &body.username,
        &body.email,
        &body.password,
        &state.jwt_secret,
    )
    .await?;

    Ok(Json(AuthResponse {
        user_id: result.user_id,
        token: result.token,
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let result = rusteze_auth::session::login(
        &state.db,
        &body.email,
        &body.password,
        &state.jwt_secret,
    )
    .await?;

    Ok(Json(AuthResponse {
        user_id: result.user_id,
        token: result.token,
    }))
}
