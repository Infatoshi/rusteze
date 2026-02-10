pub mod password;
pub mod session;
pub mod token;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account not found")]
    AccountNotFound,
    #[error("token expired")]
    TokenExpired,
    #[error("invalid token")]
    InvalidToken,
    #[error("mfa required")]
    MfaRequired,
    #[error("invalid mfa code")]
    InvalidMfaCode,
    #[error("database error: {0}")]
    Db(#[from] rusteze_db::DbError),
}

pub type AuthResult<T> = Result<T, AuthError>;
