use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Serialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub banner_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct SelfUserResponse {
    pub id: Uuid,
    pub username: String,
    pub discriminator: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub bio: Option<String>,
    pub banner_url: Option<String>,
    pub flags: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub banner_url: Option<String>,
}

/// GET /users/@me — get the authenticated user's full profile
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<SelfUserResponse>, ApiError> {
    let u = rusteze_db::users::find_by_id(&state.db, user.0).await?;
    Ok(Json(SelfUserResponse {
        id: u.id,
        username: u.username,
        discriminator: u.discriminator,
        display_name: u.display_name,
        avatar_url: u.avatar_url,
        email: u.email,
        phone: u.phone,
        bio: u.bio,
        banner_url: u.banner_url,
        flags: u.flags,
        created_at: u.created_at,
        updated_at: u.updated_at,
    }))
}

/// PATCH /users/@me — update the authenticated user's profile
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> Result<Json<SelfUserResponse>, ApiError> {
    let u = rusteze_db::users::update_profile(
        &state.db,
        user.0,
        body.display_name.as_ref().map(|s| Some(s.as_str())),
        body.avatar_url.as_ref().map(|s| Some(s.as_str())),
        body.bio.as_ref().map(|s| Some(s.as_str())),
        body.banner_url.as_ref().map(|s| Some(s.as_str())),
    )
    .await?;

    Ok(Json(SelfUserResponse {
        id: u.id,
        username: u.username,
        discriminator: u.discriminator,
        display_name: u.display_name,
        avatar_url: u.avatar_url,
        email: u.email,
        phone: u.phone,
        bio: u.bio,
        banner_url: u.banner_url,
        flags: u.flags,
        created_at: u.created_at,
        updated_at: u.updated_at,
    }))
}

/// GET /users/{user_id} — get another user's public profile
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    _user: AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserProfileResponse>, ApiError> {
    let u = rusteze_db::users::find_by_id(&state.db, user_id).await?;
    Ok(Json(UserProfileResponse {
        id: u.id,
        username: u.username,
        discriminator: u.discriminator,
        display_name: u.display_name,
        avatar_url: u.avatar_url,
        bio: u.bio,
        banner_url: u.banner_url,
        created_at: u.created_at,
    }))
}
