use std::sync::Arc;

use axum::{Json, extract::{Path, State}};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::ApiError, extract::AuthUser, state::AppState};

#[derive(Serialize)]
pub struct InviteResponse {
    pub code: String,
    pub server_id: Uuid,
}

fn generate_invite_code() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    (0..8)
        .map(|_| {
            let idx: usize = rng.random_range(0..36);
            if idx < 10 {
                (b'0' + idx as u8) as char
            } else {
                (b'a' + (idx - 10) as u8) as char
            }
        })
        .collect()
}

pub async fn create_invite(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(server_id): Path<Uuid>,
) -> Result<Json<InviteResponse>, ApiError> {
    if !rusteze_db::members::is_member(&state.db, server_id, user.0).await? {
        return Err(ApiError {
            status: axum::http::StatusCode::FORBIDDEN,
            message: "not a member of this server".into(),
        });
    }

    let code = generate_invite_code();
    let invite = rusteze_db::invites::create_invite(&state.db, server_id, user.0, &code).await?;

    Ok(Json(InviteResponse {
        code: invite.code,
        server_id: invite.server_id,
    }))
}

pub async fn join_invite(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Path(code): Path<String>,
) -> Result<Json<rusteze_db::members::MemberRow>, ApiError> {
    let invite = rusteze_db::invites::use_invite(&state.db, &code).await?;
    let member = rusteze_db::members::add_member(&state.db, invite.server_id, user.0).await?;
    Ok(Json(member))
}
