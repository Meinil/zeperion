use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::domain::background::{BackgroundImage, BackgroundSource};

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<BackgroundImage>> {
    let rows = sqlx::query(
        "SELECT id, source_type, source_value, local_path, is_active, created_at
         FROM background_images
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(BackgroundImage {
            id: row.try_get("id")?,
            source_type: match row.try_get::<String, _>("source_type")?.as_str() {
                "url" => BackgroundSource::Url,
                _ => BackgroundSource::LocalFile,
            },
            source_value: row.try_get("source_value")?,
            local_path: row.try_get("local_path")?,
            is_active: row.try_get::<i64, _>("is_active")? == 1,
            created_at: row
                .try_get::<String, _>("created_at")?
                .parse::<DateTime<Utc>>()?,
        });
    }

    Ok(items)
}

pub async fn insert(pool: &SqlitePool, item: &BackgroundImage) -> Result<()> {
    sqlx::query(
        "INSERT INTO background_images
        (id, source_type, source_value, local_path, is_active, created_at)
        VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&item.id)
    .bind(item.source_type.as_str())
    .bind(&item.source_value)
    .bind(&item.local_path)
    .bind(if item.is_active { 1 } else { 0 })
    .bind(item.created_at.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_active(pool: &SqlitePool, background_id: &str) -> Result<()> {
    let already_active: Option<i64> =
        sqlx::query_scalar("SELECT is_active FROM background_images WHERE id = ?")
            .bind(background_id)
            .fetch_optional(pool)
            .await?;
    if already_active == Some(1) {
        return Ok(());
    }

    sqlx::query("UPDATE background_images SET is_active = 0")
        .execute(pool)
        .await?;
    sqlx::query("UPDATE background_images SET is_active = 1 WHERE id = ?")
        .bind(background_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, background_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM background_images WHERE id = ?")
        .bind(background_id)
        .execute(pool)
        .await?;
    Ok(())
}
