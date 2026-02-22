use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct OAuthAccountRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_id: String,
    pub email: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Find an existing OAuth account by provider + provider_id.
pub async fn find_by_provider(
    pool: &PgPool,
    provider: &str,
    provider_id: &str,
) -> DbResult<OAuthAccountRow> {
    let row: Option<OAuthAccountRow> = sqlx::query_as(
        "SELECT * FROM oauth_accounts WHERE provider = $1 AND provider_id = $2",
    )
    .bind(provider)
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    row.ok_or(crate::DbError::NotFound)
}

/// Link an OAuth account to an existing user.
pub async fn link_account(
    pool: &PgPool,
    user_id: Uuid,
    provider: &str,
    provider_id: &str,
    email: Option<&str>,
    access_token: Option<&str>,
    refresh_token: Option<&str>,
) -> DbResult<OAuthAccountRow> {
    let id = Uuid::now_v7();

    let row: OAuthAccountRow = sqlx::query_as(
        "INSERT INTO oauth_accounts (id, user_id, provider, provider_id, email, access_token, refresh_token) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
    )
    .bind(id)
    .bind(user_id)
    .bind(provider)
    .bind(provider_id)
    .bind(email)
    .bind(access_token)
    .bind(refresh_token)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Get all OAuth accounts linked to a user.
pub async fn user_accounts(pool: &PgPool, user_id: Uuid) -> DbResult<Vec<OAuthAccountRow>> {
    let rows: Vec<OAuthAccountRow> =
        sqlx::query_as("SELECT * FROM oauth_accounts WHERE user_id = $1 ORDER BY created_at")
            .bind(user_id)
            .fetch_all(pool)
            .await?;

    Ok(rows)
}

/// Remove an OAuth account link.
pub async fn unlink_account(pool: &PgPool, user_id: Uuid, provider: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM oauth_accounts WHERE user_id = $1 AND provider = $2")
        .bind(user_id)
        .bind(provider)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
}
