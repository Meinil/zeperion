use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::domain::download::{DownloadStatus, DownloadTaskRecord};

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<DownloadTaskRecord>> {
    let rows = sqlx::query(
        "SELECT id, package_id, package_name, task_type, status, downloaded_bytes, total_bytes, target_path, metadata_json, updated_at
         FROM download_tasks
         WHERE task_type = 'core_download'
         ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut tasks = Vec::with_capacity(rows.len());
    for row in rows {
        tasks.push(DownloadTaskRecord {
            id: row.try_get("id")?,
            package_id: row.try_get("package_id")?,
            package_name: row.try_get("package_name")?,
            task_type: row.try_get("task_type")?,
            status: DownloadStatus::from_str(&row.try_get::<String, _>("status")?),
            downloaded_bytes: row.try_get("downloaded_bytes")?,
            total_bytes: row.try_get("total_bytes")?,
            target_path: row.try_get("target_path")?,
            metadata_json: row.try_get("metadata_json")?,
            updated_at: row
                .try_get::<String, _>("updated_at")?
                .parse::<DateTime<Utc>>()?,
        });
    }

    Ok(tasks)
}

pub async fn upsert(pool: &SqlitePool, task: &DownloadTaskRecord) -> Result<()> {
    if task.task_type == "core_download" {
        sqlx::query(
            "DELETE FROM download_tasks WHERE package_id = ? AND task_type <> 'core_download'",
        )
        .bind(&task.package_id)
        .execute(pool)
        .await?;
    }

    sqlx::query(
        "INSERT INTO download_tasks
        (id, package_id, package_name, task_type, status, downloaded_bytes, total_bytes, target_path, metadata_json, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            package_id = excluded.package_id,
            package_name = excluded.package_name,
            task_type = excluded.task_type,
            status = excluded.status,
            downloaded_bytes = excluded.downloaded_bytes,
            total_bytes = excluded.total_bytes,
            target_path = excluded.target_path,
            metadata_json = excluded.metadata_json,
            updated_at = excluded.updated_at",
    )
    .bind(&task.id)
    .bind(&task.package_id)
    .bind(&task.package_name)
    .bind(&task.task_type)
    .bind(task.status.as_str())
    .bind(task.downloaded_bytes)
    .bind(task.total_bytes)
    .bind(&task.target_path)
    .bind(&task.metadata_json)
    .bind(task.updated_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_by_package_id(pool: &SqlitePool, package_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM download_tasks WHERE package_id = ?")
        .bind(package_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_by_id(pool: &SqlitePool, task_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM download_tasks WHERE id = ?")
        .bind(task_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_non_core_by_package_id(pool: &SqlitePool, package_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM download_tasks WHERE package_id = ? AND task_type <> 'core_download'")
        .bind(package_id)
        .execute(pool)
        .await?;
    Ok(())
}
