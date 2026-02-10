use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AuthResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,       // user id
    pub sid: Uuid,       // session id
    pub exp: i64,        // expiry
    pub iat: i64,        // issued at
}

/// Create a JWT for a user session.
pub fn create_token(user_id: Uuid, session_id: Uuid, secret: &str) -> AuthResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        sid: session_id,
        exp: (now + Duration::days(30)).timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| crate::AuthError::InvalidToken)
}

/// Validate a JWT and return the claims.
pub fn validate_token(token: &str, secret: &str) -> AuthResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => crate::AuthError::TokenExpired,
        _ => crate::AuthError::InvalidToken,
    })
}
