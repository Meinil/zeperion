pub mod accounts;
pub mod backgrounds;
pub mod custom_mirrors;
pub mod download_tasks;
pub mod java;
pub mod launch_history;
pub mod messages;
pub mod settings;
pub mod versions;
pub mod window_prefs;

use anyhow::Result;
use sqlx::{Executor, Row, SqlitePool, migrate::MigrateError, sqlite::SqlitePoolOptions};
use uuid::Uuid;

use crate::domain::account;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
const INIT_SQL: &str = include_str!("../../../migrations/init.sql");

pub async fn connect(db_path: &std::path::Path) -> Result<SqlitePool> {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    match MIGRATOR.run(pool).await {
        Ok(()) => {}
        Err(MigrateError::VersionMismatch(1)) => {
            repair_init_migration_checksum(pool).await?;
            MIGRATOR.run(pool).await?;
        }
        Err(error) => return Err(error.into()),
    }

    ensure_init_schema(pool).await?;
    ensure_accounts_schema(pool).await?;

    Ok(())
}

async fn repair_init_migration_checksum(pool: &SqlitePool) -> Result<()> {
    let Some(migration) = MIGRATOR.iter().find(|migration| migration.version == 1) else {
        return Ok(());
    };

    sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = 1")
        .bind(migration.checksum.as_ref())
        .execute(pool)
        .await?;

    Ok(())
}

async fn ensure_init_schema(pool: &SqlitePool) -> Result<()> {
    let exists: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'custom_mirrors'",
    )
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        pool.execute(INIT_SQL).await?;
    }

    Ok(())
}

async fn ensure_accounts_schema(pool: &SqlitePool) -> Result<()> {
    let additions = [
        ("uuid", "ALTER TABLE accounts ADD COLUMN uuid TEXT"),
        (
            "refresh_token",
            "ALTER TABLE accounts ADD COLUMN refresh_token TEXT",
        ),
        (
            "client_token",
            "ALTER TABLE accounts ADD COLUMN client_token TEXT",
        ),
        (
            "auth_server_url",
            "ALTER TABLE accounts ADD COLUMN auth_server_url TEXT",
        ),
        (
            "authlib_metadata_b64",
            "ALTER TABLE accounts ADD COLUMN authlib_metadata_b64 TEXT",
        ),
        (
            "oauth_flow",
            "ALTER TABLE accounts ADD COLUMN oauth_flow TEXT",
        ),
        (
            "skin_mode",
            "ALTER TABLE accounts ADD COLUMN skin_mode TEXT",
        ),
        (
            "skin_reference",
            "ALTER TABLE accounts ADD COLUMN skin_reference TEXT",
        ),
        (
            "avatar_url",
            "ALTER TABLE accounts ADD COLUMN avatar_url TEXT",
        ),
        (
            "updated_at",
            "ALTER TABLE accounts ADD COLUMN updated_at TEXT",
        ),
        (
            "last_validated_at",
            "ALTER TABLE accounts ADD COLUMN last_validated_at TEXT",
        ),
    ];

    for (column, statement) in additions {
        if !column_exists(pool, "accounts", column).await? {
            sqlx::query(statement).execute(pool).await?;
        }
    }

    let rows = sqlx::query(
        "SELECT id, username, login_type, uuid, skin_mode, skin_reference, created_at, updated_at
         FROM accounts",
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        let id: String = row.try_get("id")?;
        let username: String = row.try_get("username")?;
        let login_type: String = row.try_get("login_type")?;
        let existing_uuid: Option<String> = row.try_get("uuid")?;
        let created_at: String = row.try_get("created_at")?;
        let updated_at: Option<String> = row.try_get("updated_at")?;
        let skin_mode: Option<String> = row.try_get("skin_mode")?;
        let skin_reference: Option<String> = row.try_get("skin_reference")?;

        let uuid = existing_uuid
            .as_deref()
            .and_then(account::normalize_uuid)
            .unwrap_or_else(|| {
                if login_type == "offline" {
                    account::generate_offline_uuid_for_skin_reference(
                        account::default_offline_skin_reference(),
                    )
                } else {
                    Uuid::new_v4().simple().to_string()
                }
            });

        let next_skin_mode = if login_type == "offline" {
            Some(
                skin_mode
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| "default".to_string()),
            )
        } else {
            skin_mode
        };

        let next_skin_reference = if login_type == "offline" {
            Some(
                skin_reference
                    .filter(|value| !value.trim().is_empty())
                    .unwrap_or_else(|| account::default_offline_skin_reference().to_string()),
            )
        } else {
            skin_reference
        };

        sqlx::query(
            "UPDATE accounts
             SET uuid = ?, skin_mode = ?, skin_reference = ?, updated_at = COALESCE(updated_at, ?)
             WHERE id = ?",
        )
        .bind(uuid)
        .bind(next_skin_mode)
        .bind(next_skin_reference)
        .bind(updated_at.unwrap_or(created_at))
        .bind(id)
        .execute(pool)
        .await?;

        let _ = username;
    }

    Ok(())
}

async fn column_exists(pool: &SqlitePool, table: &str, column: &str) -> Result<bool> {
    let pragma = format!("PRAGMA table_info({table})");
    let rows = sqlx::query(&pragma).fetch_all(pool).await?;
    Ok(rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>("name").ok())
        .any(|name| name == column))
}
