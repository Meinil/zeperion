use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::domain::account::{Account, LoginType, OAuthFlow, SkinMode};

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<Account>> {
    let rows = sqlx::query(
        "SELECT id, username, uuid, login_type, access_token, refresh_token, client_token,
                auth_server_url, authlib_metadata_b64, oauth_flow, skin_mode, skin_reference,
                avatar_url, selected, created_at, updated_at, last_validated_at
         FROM accounts
         ORDER BY selected DESC, created_at ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut accounts = Vec::with_capacity(rows.len());
    for row in rows {
        accounts.push(Account {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            uuid: row.try_get("uuid")?,
            login_type: LoginType::from_str(&row.try_get::<String, _>("login_type")?),
            access_token: row.try_get("access_token")?,
            refresh_token: row.try_get("refresh_token")?,
            client_token: row.try_get("client_token")?,
            auth_server_url: row.try_get("auth_server_url")?,
            authlib_metadata_b64: row.try_get("authlib_metadata_b64")?,
            oauth_flow: OAuthFlow::from_optional(
                row.try_get::<Option<String>, _>("oauth_flow")?.as_deref(),
            ),
            skin_mode: SkinMode::from_optional(
                row.try_get::<Option<String>, _>("skin_mode")?.as_deref(),
            ),
            skin_reference: row.try_get("skin_reference")?,
            avatar_url: row.try_get("avatar_url")?,
            selected: row.try_get::<i64, _>("selected")? == 1,
            created_at: row
                .try_get::<String, _>("created_at")?
                .parse::<DateTime<Utc>>()?,
            updated_at: row
                .try_get::<String, _>("updated_at")?
                .parse::<DateTime<Utc>>()?,
            last_validated_at: row
                .try_get::<Option<String>, _>("last_validated_at")?
                .map(|value| value.parse::<DateTime<Utc>>())
                .transpose()?,
        });
    }

    Ok(accounts)
}

pub async fn insert(pool: &SqlitePool, account: &Account) -> Result<()> {
    if account.selected {
        sqlx::query("UPDATE accounts SET selected = 0")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        "INSERT INTO accounts (
            id, username, uuid, login_type, access_token, refresh_token, client_token,
            auth_server_url, authlib_metadata_b64, oauth_flow, skin_mode, skin_reference,
            avatar_url, selected, created_at, updated_at, last_validated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&account.id)
    .bind(&account.username)
    .bind(&account.uuid)
    .bind(account.login_type.as_str())
    .bind(&account.access_token)
    .bind(&account.refresh_token)
    .bind(&account.client_token)
    .bind(&account.auth_server_url)
    .bind(&account.authlib_metadata_b64)
    .bind(account.oauth_flow_str())
    .bind(account.skin_mode_str())
    .bind(&account.skin_reference)
    .bind(&account.avatar_url)
    .bind(if account.selected { 1 } else { 0 })
    .bind(account.created_at.to_rfc3339())
    .bind(account.updated_at.to_rfc3339())
    .bind(account.last_validated_at.map(|value| value.to_rfc3339()))
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn upsert(pool: &SqlitePool, account: &Account) -> Result<()> {
    if account.selected {
        sqlx::query("UPDATE accounts SET selected = 0")
            .execute(pool)
            .await?;
    }

    sqlx::query(
        "INSERT OR REPLACE INTO accounts (
            id, username, uuid, login_type, access_token, refresh_token, client_token,
            auth_server_url, authlib_metadata_b64, oauth_flow, skin_mode, skin_reference,
            avatar_url, selected, created_at, updated_at, last_validated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&account.id)
    .bind(&account.username)
    .bind(&account.uuid)
    .bind(account.login_type.as_str())
    .bind(&account.access_token)
    .bind(&account.refresh_token)
    .bind(&account.client_token)
    .bind(&account.auth_server_url)
    .bind(&account.authlib_metadata_b64)
    .bind(account.oauth_flow_str())
    .bind(account.skin_mode_str())
    .bind(&account.skin_reference)
    .bind(&account.avatar_url)
    .bind(if account.selected { 1 } else { 0 })
    .bind(account.created_at.to_rfc3339())
    .bind(account.updated_at.to_rfc3339())
    .bind(account.last_validated_at.map(|value| value.to_rfc3339()))
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_tokens(
    pool: &SqlitePool,
    account_id: &str,
    access_token: Option<&str>,
    refresh_token: Option<&str>,
    last_validated_at: Option<DateTime<Utc>>,
) -> Result<()> {
    sqlx::query(
        "UPDATE accounts
         SET access_token = ?, refresh_token = ?, updated_at = ?, last_validated_at = ?
         WHERE id = ?",
    )
    .bind(access_token)
    .bind(refresh_token)
    .bind(Utc::now().to_rfc3339())
    .bind(last_validated_at.map(|value| value.to_rfc3339()))
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_selected(pool: &SqlitePool, account_id: &str) -> Result<()> {
    let already_selected: Option<i64> =
        sqlx::query_scalar("SELECT selected FROM accounts WHERE id = ?")
            .bind(account_id)
            .fetch_optional(pool)
            .await?;
    if already_selected == Some(1) {
        return Ok(());
    }

    sqlx::query("UPDATE accounts SET selected = 0")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE accounts SET selected = 1 WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, account_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await?;
    Ok(())
}
