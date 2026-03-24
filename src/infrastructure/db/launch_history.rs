use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

#[derive(Clone, Debug, PartialEq)]
pub struct LaunchHistoryRecord {
    pub id: String,
    pub version_name: String,
    pub account_name: String,
    pub java_path: String,
    pub command_line: String,
    pub log_path: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub error_message: Option<String>,
    pub launched_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

pub async fn upsert(pool: &SqlitePool, record: &LaunchHistoryRecord) -> Result<()> {
    sqlx::query(
        "INSERT INTO launch_history
        (id, version_name, account_name, java_path, command_line, log_path, status, exit_code, error_message, launched_at, finished_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            version_name = excluded.version_name,
            account_name = excluded.account_name,
            java_path = excluded.java_path,
            command_line = excluded.command_line,
            log_path = excluded.log_path,
            status = excluded.status,
            exit_code = excluded.exit_code,
            error_message = excluded.error_message,
            launched_at = excluded.launched_at,
            finished_at = excluded.finished_at",
    )
    .bind(&record.id)
    .bind(&record.version_name)
    .bind(&record.account_name)
    .bind(&record.java_path)
    .bind(&record.command_line)
    .bind(&record.log_path)
    .bind(&record.status)
    .bind(record.exit_code)
    .bind(&record.error_message)
    .bind(record.launched_at.to_rfc3339())
    .bind(record.finished_at.map(|value| value.to_rfc3339()))
    .execute(pool)
    .await?;
    Ok(())
}
