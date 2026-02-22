use sqlx::PgPool;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::AuthResult;

/// Generate a TOTP secret and store it (disabled until verified).
pub async fn setup_totp(pool: &PgPool, user_id: Uuid, issuer: &str) -> AuthResult<String> {
    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();

    // Fetch user for account name
    let user = rusteze_db::users::find_by_id(pool, user_id).await?;
    let account = user.email.unwrap_or(user.username);

    // Store the secret (not yet enabled)
    sqlx::query(
        r#"
        INSERT INTO mfa_secrets (user_id, totp_secret, enabled)
        VALUES ($1, $2, false)
        ON CONFLICT (user_id) DO UPDATE SET totp_secret = $2, enabled = false
        "#,
    )
    .bind(user_id)
    .bind(&secret_base32)
    .execute(pool)
    .await
    .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    // Build the otpauth:// URI for QR code generation
    let totp = TOTP::new(
        Algorithm::SHA1, 6, 1, 30,
        secret.to_bytes().unwrap(),
        Some(issuer.to_string()),
        account,
    )
    .map_err(|_| crate::AuthError::InvalidMfaCode)?;
    let uri = totp.get_url();

    Ok(uri)
}

/// Verify a TOTP code and enable MFA if valid. Returns backup codes.
pub async fn verify_and_enable(
    pool: &PgPool,
    user_id: Uuid,
    code: &str,
) -> AuthResult<Vec<String>> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT totp_secret FROM mfa_secrets WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    let (secret_base32,) = row.ok_or(crate::AuthError::InvalidMfaCode)?;
    let secret = Secret::Encoded(secret_base32)
        .to_bytes()
        .map_err(|_| crate::AuthError::InvalidMfaCode)?;

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, None, String::new())
        .map_err(|_| crate::AuthError::InvalidMfaCode)?;

    if !totp.check_current(code).map_err(|_| crate::AuthError::InvalidMfaCode)? {
        return Err(crate::AuthError::InvalidMfaCode);
    }

    // Generate backup codes (ThreadRng is !Send, so scope it before the await)
    let backup_codes: Vec<String> = {
        use rand::Rng;
        let mut rng = rand::rng();
        (0..8)
            .map(|_| {
                let code: u32 = rng.random_range(10000000..99999999);
                format!("{code}")
            })
            .collect()
    };

    let codes_arr: Vec<&str> = backup_codes.iter().map(|s| s.as_str()).collect();

    // Enable MFA and store backup codes
    sqlx::query(
        "UPDATE mfa_secrets SET enabled = true, backup_codes = $1 WHERE user_id = $2",
    )
    .bind(&codes_arr)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    Ok(backup_codes)
}

/// Validate a TOTP code during login (or a backup code).
pub async fn validate_code(pool: &PgPool, user_id: Uuid, code: &str) -> AuthResult<()> {
    let row: Option<(String, bool, Vec<String>)> = sqlx::query_as(
        "SELECT totp_secret, enabled, backup_codes FROM mfa_secrets WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    let (secret_base32, enabled, backup_codes) =
        row.ok_or(crate::AuthError::InvalidMfaCode)?;

    if !enabled {
        return Ok(()); // MFA not enabled, skip
    }

    let secret = Secret::Encoded(secret_base32)
        .to_bytes()
        .map_err(|_| crate::AuthError::InvalidMfaCode)?;

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret, None, String::new())
        .map_err(|_| crate::AuthError::InvalidMfaCode)?;

    // Try TOTP first
    if totp.check_current(code).unwrap_or(false) {
        return Ok(());
    }

    // Try backup code
    if backup_codes.contains(&code.to_string()) {
        // Remove the used backup code
        let remaining: Vec<&str> = backup_codes
            .iter()
            .filter(|c| c.as_str() != code)
            .map(|c| c.as_str())
            .collect();

        sqlx::query("UPDATE mfa_secrets SET backup_codes = $1 WHERE user_id = $2")
            .bind(&remaining)
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

        return Ok(());
    }

    Err(crate::AuthError::InvalidMfaCode)
}

/// Check if MFA is enabled for a user.
pub async fn is_enabled(pool: &PgPool, user_id: Uuid) -> AuthResult<bool> {
    let row: Option<(bool,)> = sqlx::query_as(
        "SELECT enabled FROM mfa_secrets WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    Ok(row.map(|(e,)| e).unwrap_or(false))
}

/// Disable MFA for a user.
pub async fn disable(pool: &PgPool, user_id: Uuid) -> AuthResult<()> {
    sqlx::query("DELETE FROM mfa_secrets WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| crate::AuthError::Db(rusteze_db::DbError::Sqlx(e)))?;

    Ok(())
}
