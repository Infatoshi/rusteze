use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({
                "error": self.message
            })),
        )
            .into_response()
    }
}

impl From<rusteze_db::DbError> for ApiError {
    fn from(e: rusteze_db::DbError) -> Self {
        match e {
            rusteze_db::DbError::NotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                message: "not found".into(),
            },
            rusteze_db::DbError::AlreadyExists => ApiError {
                status: StatusCode::CONFLICT,
                message: "already exists".into(),
            },
            _ => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "internal error".into(),
            },
        }
    }
}

impl From<rusteze_auth::AuthError> for ApiError {
    fn from(e: rusteze_auth::AuthError) -> Self {
        match e {
            rusteze_auth::AuthError::InvalidCredentials => ApiError {
                status: StatusCode::UNAUTHORIZED,
                message: "invalid credentials".into(),
            },
            rusteze_auth::AuthError::AccountNotFound => ApiError {
                status: StatusCode::NOT_FOUND,
                message: "account not found".into(),
            },
            rusteze_auth::AuthError::TokenExpired | rusteze_auth::AuthError::InvalidToken => {
                ApiError {
                    status: StatusCode::UNAUTHORIZED,
                    message: "invalid or expired token".into(),
                }
            }
            _ => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: "internal error".into(),
            },
        }
    }
}
