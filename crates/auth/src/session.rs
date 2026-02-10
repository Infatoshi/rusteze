use sqlx::PgPool;
use uuid::Uuid;

use crate::{password, token, AuthResult};

pub struct LoginResult {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub token: String,
}

/// Register a new user.
pub async fn register(
    pool: &PgPool,
    username: &str,
    email: &str,
    password: &str,
    jwt_secret: &str,
) -> AuthResult<LoginResult> {
    let hash = password::hash_password(password)?;
    let user = rusteze_db::users::create_user(pool, username, email, &hash).await?;
    let session_id = Uuid::now_v7();

    let token_str = token::create_token(user.id, session_id, jwt_secret)?;
    let token_hash = sha256_hex(&token_str);

    sqlx::query("INSERT INTO sessions (id, user_id, token_hash) VALUES ($1, $2, $3)")
        .bind(session_id)
        .bind(user.id)
        .bind(&token_hash)
        .execute(pool)
        .await
        .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    Ok(LoginResult {
        user_id: user.id,
        session_id,
        token: token_str,
    })
}

/// Log in with email and password.
pub async fn login(
    pool: &PgPool,
    email: &str,
    password_raw: &str,
    jwt_secret: &str,
) -> AuthResult<LoginResult> {
    let user = rusteze_db::users::find_by_email(pool, email)
        .await
        .map_err(|_| crate::AuthError::AccountNotFound)?;

    password::verify_password(password_raw, &user.password_hash)?;

    let session_id = Uuid::now_v7();
    let token_str = token::create_token(user.id, session_id, jwt_secret)?;
    let token_hash = sha256_hex(&token_str);

    sqlx::query("INSERT INTO sessions (id, user_id, token_hash) VALUES ($1, $2, $3)")
        .bind(session_id)
        .bind(user.id)
        .bind(&token_hash)
        .execute(pool)
        .await
        .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    Ok(LoginResult {
        user_id: user.id,
        session_id,
        token: token_str,
    })
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
