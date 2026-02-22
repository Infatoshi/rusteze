use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{error::ApiError, extract::AuthUser, state::AppState};

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
    pub mfa_code: Option<String>,
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

    // Check if MFA is enabled
    if rusteze_auth::mfa::is_enabled(&state.db, result.user_id).await? {
        match body.mfa_code {
            Some(code) => {
                rusteze_auth::mfa::validate_code(&state.db, result.user_id, &code).await?;
            }
            None => {
                return Err(ApiError {
                    status: axum::http::StatusCode::FORBIDDEN,
                    message: "mfa_required".into(),
                });
            }
        }
    }

    Ok(Json(AuthResponse {
        user_id: result.user_id,
        token: result.token,
    }))
}

// --- MFA endpoints ---

#[derive(Serialize)]
pub struct MfaSetupResponse {
    pub otpauth_uri: String,
}

#[derive(Deserialize)]
pub struct MfaVerifyRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct MfaVerifyResponse {
    pub backup_codes: Vec<String>,
}

/// POST /auth/mfa/setup — start MFA setup, returns otpauth URI for QR code
pub async fn mfa_setup(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<MfaSetupResponse>, ApiError> {
    let uri = rusteze_auth::mfa::setup_totp(&state.db, user.0, "rusteze").await?;
    Ok(Json(MfaSetupResponse { otpauth_uri: uri }))
}

/// POST /auth/mfa/verify — verify TOTP code and enable MFA
pub async fn mfa_verify(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
    Json(body): Json<MfaVerifyRequest>,
) -> Result<Json<MfaVerifyResponse>, ApiError> {
    let backup_codes = rusteze_auth::mfa::verify_and_enable(&state.db, user.0, &body.code).await?;
    Ok(Json(MfaVerifyResponse { backup_codes }))
}

/// DELETE /auth/mfa — disable MFA
pub async fn mfa_disable(
    State(state): State<Arc<AppState>>,
    user: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    rusteze_auth::mfa::disable(&state.db, user.0).await?;
    Ok(Json(serde_json::json!({"disabled": true})))
}

// --- OAuth endpoints ---

#[derive(Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub provider: String,
}

#[derive(Serialize)]
pub struct OAuthUrlResponse {
    pub url: String,
}

/// GET /auth/oauth/google — get Google OAuth URL
pub async fn oauth_google_url(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<OAuthUrlResponse>, ApiError> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").map_err(|_| ApiError {
        status: axum::http::StatusCode::SERVICE_UNAVAILABLE,
        message: "google oauth not configured".into(),
    })?;
    let redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:14702/auth/oauth/callback".into());

    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=openid%20email%20profile&access_type=offline",
        client_id, redirect_uri
    );

    Ok(Json(OAuthUrlResponse { url }))
}

/// GET /auth/oauth/github — get GitHub OAuth URL
pub async fn oauth_github_url(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<OAuthUrlResponse>, ApiError> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").map_err(|_| ApiError {
        status: axum::http::StatusCode::SERVICE_UNAVAILABLE,
        message: "github oauth not configured".into(),
    })?;
    let redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:14702/auth/oauth/callback".into());

    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email",
        client_id, redirect_uri
    );

    Ok(Json(OAuthUrlResponse { url }))
}

/// POST /auth/oauth/callback — handle OAuth callback, create/login user
pub async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Json(body): Json<OAuthCallbackRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let (provider_id, email, username) = match body.provider.as_str() {
        "google" => exchange_google_token(&body.code).await?,
        "github" => exchange_github_token(&body.code).await?,
        _ => {
            return Err(ApiError {
                status: axum::http::StatusCode::BAD_REQUEST,
                message: "unsupported provider".into(),
            })
        }
    };

    // Check if OAuth account already linked
    let user_id = match rusteze_db::oauth::find_by_provider(&state.db, &body.provider, &provider_id).await {
        Ok(oauth) => oauth.user_id,
        Err(rusteze_db::DbError::NotFound) => {
            // Check if a user with this email exists
            let user = match rusteze_db::users::find_by_email(&state.db, &email).await {
                Ok(u) => u,
                Err(rusteze_db::DbError::NotFound) => {
                    // Create new user
                    rusteze_db::users::create_oauth_user(&state.db, &username, &email).await?
                }
                Err(e) => return Err(e.into()),
            };

            // Link the OAuth account
            rusteze_db::oauth::link_account(
                &state.db,
                user.id,
                &body.provider,
                &provider_id,
                Some(&email),
                None,
                None,
            )
            .await?;

            user.id
        }
        Err(e) => return Err(e.into()),
    };

    // Create a session
    let session_id = uuid::Uuid::now_v7();
    let token = rusteze_auth::token::create_token(user_id, session_id, &state.jwt_secret)?;
    let token_hash = sha256_hex(&token);

    sqlx::query("INSERT INTO sessions (id, user_id, token_hash) VALUES ($1, $2, $3)")
        .bind(session_id)
        .bind(user_id)
        .bind(&token_hash)
        .execute(&state.db)
        .await
        .map_err(|e| ApiError {
            status: axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("session error: {e}"),
        })?;

    Ok(Json(AuthResponse {
        user_id,
        token,
    }))
}

/// Exchange a Google auth code for user info. Returns (provider_id, email, username).
async fn exchange_google_token(code: &str) -> Result<(String, String, String), ApiError> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
    let redirect_uri = std::env::var("OAUTH_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:14702/auth/oauth/callback".into());

    let client = reqwest::Client::new();

    // Exchange code for tokens
    let token_res = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", &redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: format!("google token exchange failed: {e}"),
        })?;

    let token_json: serde_json::Value = token_res.json().await.map_err(|e| ApiError {
        status: axum::http::StatusCode::BAD_GATEWAY,
        message: format!("google token parse failed: {e}"),
    })?;

    let access_token = token_json["access_token"]
        .as_str()
        .ok_or(ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: "missing access_token from google".into(),
        })?;

    // Get user info
    let user_res = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: format!("google userinfo failed: {e}"),
        })?;

    let user_json: serde_json::Value = user_res.json().await.map_err(|e| ApiError {
        status: axum::http::StatusCode::BAD_GATEWAY,
        message: format!("google userinfo parse failed: {e}"),
    })?;

    let provider_id = user_json["id"].as_str().unwrap_or("").to_string();
    let email = user_json["email"].as_str().unwrap_or("").to_string();
    let name = user_json["name"]
        .as_str()
        .unwrap_or(&email)
        .to_string();

    Ok((provider_id, email, name))
}

/// Exchange a GitHub auth code for user info. Returns (provider_id, email, username).
async fn exchange_github_token(code: &str) -> Result<(String, String, String), ApiError> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();

    let client = reqwest::Client::new();

    // Exchange code for access token
    let token_res = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("code", code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
        ])
        .send()
        .await
        .map_err(|e| ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: format!("github token exchange failed: {e}"),
        })?;

    let token_json: serde_json::Value = token_res.json().await.map_err(|e| ApiError {
        status: axum::http::StatusCode::BAD_GATEWAY,
        message: format!("github token parse failed: {e}"),
    })?;

    let access_token = token_json["access_token"]
        .as_str()
        .ok_or(ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: "missing access_token from github".into(),
        })?;

    // Get user info
    let user_res = client
        .get("https://api.github.com/user")
        .header("User-Agent", "rusteze")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: format!("github user fetch failed: {e}"),
        })?;

    let user_json: serde_json::Value = user_res.json().await.map_err(|e| ApiError {
        status: axum::http::StatusCode::BAD_GATEWAY,
        message: format!("github user parse failed: {e}"),
    })?;

    let provider_id = user_json["id"].to_string();
    let login = user_json["login"].as_str().unwrap_or("user").to_string();

    // Get primary email
    let emails_res = client
        .get("https://api.github.com/user/emails")
        .header("User-Agent", "rusteze")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError {
            status: axum::http::StatusCode::BAD_GATEWAY,
            message: format!("github emails fetch failed: {e}"),
        })?;

    let emails_json: Vec<serde_json::Value> = emails_res.json().await.unwrap_or_default();
    let email = emails_json
        .iter()
        .find(|e| e["primary"].as_bool().unwrap_or(false))
        .and_then(|e| e["email"].as_str())
        .unwrap_or("")
        .to_string();

    Ok((provider_id, email, login))
}

fn sha256_hex(input: &str) -> String {
    use std::fmt::Write;
    let digest = <sha2::Sha256 as sha2::Digest>::digest(input.as_bytes());
    let mut s = String::with_capacity(64);
    for byte in digest {
        write!(s, "{byte:02x}").unwrap();
    }
    s
}
