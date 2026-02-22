use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::DbResult;

#[derive(Debug, serde::Serialize, FromRow)]
pub struct RoleRow {
    pub id: Uuid,
    pub server_id: Uuid,
    pub name: String,
    pub color: Option<i32>,
    pub permissions: i64,
    pub position: i32,
}

pub async fn create_role(
    pool: &PgPool,
    server_id: Uuid,
    name: &str,
    color: Option<i32>,
    permissions: i64,
) -> DbResult<RoleRow> {
    let id = Uuid::now_v7();

    // Position = max existing position + 1
    let max_pos: (Option<i32>,) = sqlx::query_as(
        "SELECT MAX(position) FROM roles WHERE server_id = $1",
    )
    .bind(server_id)
    .fetch_one(pool)
    .await?;

    let position = max_pos.0.unwrap_or(0) + 1;

    let row: RoleRow = sqlx::query_as(
        "INSERT INTO roles (id, server_id, name, color, permissions, position) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
    )
    .bind(id)
    .bind(server_id)
    .bind(name)
    .bind(color)
    .bind(permissions)
    .bind(position)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn update_role(
    pool: &PgPool,
    role_id: Uuid,
    name: Option<&str>,
    color: Option<Option<i32>>,
    permissions: Option<i64>,
) -> DbResult<RoleRow> {
    // Build dynamic update
    let current: RoleRow = sqlx::query_as("SELECT * FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_optional(pool)
        .await?
        .ok_or(crate::DbError::NotFound)?;

    let new_name = name.unwrap_or(&current.name);
    let new_color = color.unwrap_or(current.color);
    let new_perms = permissions.unwrap_or(current.permissions);

    let row: RoleRow = sqlx::query_as(
        "UPDATE roles SET name = $1, color = $2, permissions = $3 WHERE id = $4 RETURNING *",
    )
    .bind(new_name)
    .bind(new_color)
    .bind(new_perms)
    .bind(role_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn delete_role(pool: &PgPool, role_id: Uuid) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(role_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(crate::DbError::NotFound);
    }
    Ok(())
}

pub async fn fetch_server_roles(pool: &PgPool, server_id: Uuid) -> DbResult<Vec<RoleRow>> {
    let rows: Vec<RoleRow> =
        sqlx::query_as("SELECT * FROM roles WHERE server_id = $1 ORDER BY position")
            .bind(server_id)
            .fetch_all(pool)
            .await?;

    Ok(rows)
}

pub async fn assign_role(
    pool: &PgPool,
    server_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
) -> DbResult<()> {
    sqlx::query(
        "INSERT INTO member_roles (server_id, user_id, role_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(server_id)
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_role(
    pool: &PgPool,
    server_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
) -> DbResult<()> {
    sqlx::query(
        "DELETE FROM member_roles WHERE server_id = $1 AND user_id = $2 AND role_id = $3",
    )
    .bind(server_id)
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn fetch_member_roles(
    pool: &PgPool,
    server_id: Uuid,
    user_id: Uuid,
) -> DbResult<Vec<RoleRow>> {
    let rows: Vec<RoleRow> = sqlx::query_as(
        "SELECT r.* FROM roles r INNER JOIN member_roles mr ON mr.role_id = r.id WHERE mr.server_id = $1 AND mr.user_id = $2 ORDER BY r.position",
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
