use crate::features::auth::entity::{RefreshToken, User};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

const USER_COLS: &str =
    "id, email, password_hash, name, role, is_active, \
     deactivated_at, deactivated_by, deactivation_reason, created_at, updated_at";

// ── User queries ─────────────────────────────────────────────────────────────

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLS} FROM users WHERE email = $1"
    ))
    .bind(email)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(&format!(
        "SELECT {USER_COLS} FROM users WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create(
    pool: &PgPool,
    id: Uuid,
    email: &str,
    password_hash: &str,
    name: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(&format!(
        "INSERT INTO users (id, email, password_hash, name, role)
         VALUES ($1, $2, $3, $4, 'user')
         RETURNING {USER_COLS}"
    ))
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .fetch_one(pool)
    .await
}

// ── Refresh token queries ─────────────────────────────────────────────────────

pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    family_id: Uuid,
    token: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, sqlx::Error> {
    sqlx::query_as::<_, RefreshToken>(
        "INSERT INTO refresh_tokens (id, user_id, token, family_id, expires_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)
         RETURNING id, user_id, token, family_id, used, expires_at, created_at",
    )
    .bind(user_id)
    .bind(token)
    .bind(family_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn find_refresh_token(
    pool: &PgPool,
    token: &str,
) -> Result<Option<RefreshToken>, sqlx::Error> {
    sqlx::query_as::<_, RefreshToken>(
        "SELECT id, user_id, token, family_id, used, expires_at, created_at
         FROM refresh_tokens WHERE token = $1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

pub async fn delete_refresh_token(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
        .bind(token)
        .execute(pool)
        .await?;
    Ok(())
}

/// Rotate: mark old token as used and insert new one in the same family, atomically.
/// The old token is kept (not deleted) so replays can be detected as theft.
pub async fn rotate_refresh_token(
    pool: &PgPool,
    old_token: &str,
    user_id: Uuid,
    family_id: Uuid,
    new_token: &str,
    expires_at: DateTime<Utc>,
) -> Result<RefreshToken, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE refresh_tokens SET used = TRUE WHERE token = $1")
        .bind(old_token)
        .execute(&mut *tx)
        .await?;

    let rt = sqlx::query_as::<_, RefreshToken>(
        "INSERT INTO refresh_tokens (id, user_id, token, family_id, expires_at)
         VALUES (gen_random_uuid(), $1, $2, $3, $4)
         RETURNING id, user_id, token, family_id, used, expires_at, created_at",
    )
    .bind(user_id)
    .bind(new_token)
    .bind(family_id)
    .bind(expires_at)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(rt)
}

/// Revoke all tokens in a family (theft response — logs out all devices in that chain).
pub async fn delete_token_family(pool: &PgPool, family_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM refresh_tokens WHERE family_id = $1")
        .bind(family_id)
        .execute(pool)
        .await?;
    Ok(())
}
