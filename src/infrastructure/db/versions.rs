use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::domain::game_version::InstalledGameVersion;

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<InstalledGameVersion>> {
    let rows = sqlx::query(
        "SELECT id, version_name, version_type, release_time, install_dir, is_downloaded, integrity_status
         FROM game_versions
         ORDER BY release_time DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut versions = Vec::with_capacity(rows.len());
    for row in rows {
        versions.push(InstalledGameVersion {
            id: row.try_get("id")?,
            version_name: row.try_get("version_name")?,
            version_type: row.try_get("version_type")?,
            release_time: row
                .try_get::<String, _>("release_time")?
                .parse::<DateTime<Utc>>()?,
            install_dir: row.try_get("install_dir")?,
            is_downloaded: row.try_get::<i64, _>("is_downloaded")? == 1,
            integrity_status: row.try_get("integrity_status")?,
        });
    }

    Ok(versions)
}

pub async fn upsert(pool: &SqlitePool, version: &InstalledGameVersion) -> Result<()> {
    sqlx::query(
        "INSERT INTO game_versions
        (id, version_name, version_type, release_time, install_dir, is_downloaded, integrity_status)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(version_name) DO UPDATE SET
            version_type = excluded.version_type,
            release_time = excluded.release_time,
            install_dir = excluded.install_dir,
            is_downloaded = excluded.is_downloaded,
            integrity_status = excluded.integrity_status",
    )
    .bind(&version.id)
    .bind(&version.version_name)
    .bind(&version.version_type)
    .bind(version.release_time.to_rfc3339())
    .bind(&version.install_dir)
    .bind(if version.is_downloaded { 1 } else { 0 })
    .bind(&version.integrity_status)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_by_name(pool: &SqlitePool, version_name: &str) -> Result<()> {
    sqlx::query("DELETE FROM game_versions WHERE version_name = ?")
        .bind(version_name)
        .execute(pool)
        .await?;
    Ok(())
}
