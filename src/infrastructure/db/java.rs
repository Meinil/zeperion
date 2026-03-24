use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::domain::java::{JavaRuntime, JavaSource};

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<JavaRuntime>> {
    let rows = sqlx::query(
        "SELECT id, name, version, path, source, is_default, created_at
         FROM java_runtimes
         ORDER BY is_default DESC, created_at ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut runtimes = Vec::with_capacity(rows.len());
    for row in rows {
        runtimes.push(JavaRuntime {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            version: row.try_get("version")?,
            path: row.try_get("path")?,
            source: JavaSource::from_str(&row.try_get::<String, _>("source")?),
            is_default: row.try_get::<i64, _>("is_default")? == 1,
            created_at: row
                .try_get::<String, _>("created_at")?
                .parse::<DateTime<Utc>>()?,
        });
    }

    Ok(runtimes)
}

pub async fn replace_all(pool: &SqlitePool, runtimes: &[JavaRuntime]) -> Result<()> {
    sqlx::query("DELETE FROM java_runtimes")
        .execute(pool)
        .await?;
    for runtime in runtimes {
        insert(pool, runtime).await?;
    }
    Ok(())
}

pub async fn insert(pool: &SqlitePool, runtime: &JavaRuntime) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO java_runtimes
        (id, name, version, path, source, is_default, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&runtime.id)
    .bind(&runtime.name)
    .bind(&runtime.version)
    .bind(&runtime.path)
    .bind(runtime.source.as_str())
    .bind(if runtime.is_default { 1 } else { 0 })
    .bind(runtime.created_at.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_default(pool: &SqlitePool, runtime_id: &str) -> Result<()> {
    let already_default: Option<i64> =
        sqlx::query_scalar("SELECT is_default FROM java_runtimes WHERE id = ?")
            .bind(runtime_id)
            .fetch_optional(pool)
            .await?;
    if already_default == Some(1) {
        return Ok(());
    }

    sqlx::query("UPDATE java_runtimes SET is_default = 0")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE java_runtimes SET is_default = 1 WHERE id = ?")
        .bind(runtime_id)
        .execute(pool)
        .await?;
    Ok(())
}
