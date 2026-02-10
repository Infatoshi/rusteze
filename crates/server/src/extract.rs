use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use uuid::Uuid;

use crate::state::AppState;

/// Extractor that validates the Authorization header and yields the user ID.
pub struct AuthUser(pub Uuid);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = header.strip_prefix("Bearer ").unwrap_or(header);

        let claims =
            rusteze_auth::token::validate_token(token, &state.jwt_secret)
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser(claims.sub))
    }
}
