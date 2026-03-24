use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::state::AppMessage;

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<AppMessage>> {
    let rows = sqlx::query(
        "SELECT id, content, created_at, is_read
         FROM app_messages
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(AppMessage {
            id: row.try_get("id")?,
            content: row.try_get("content")?,
            created_at: row
                .try_get::<String, _>("created_at")?
                .parse::<DateTime<Utc>>()?,
            is_read: row.try_get::<i64, _>("is_read")? == 1,
        });
    }

    Ok(items)
}

pub async fn upsert(pool: &SqlitePool, message: &AppMessage) -> Result<()> {
    sqlx::query(
        "INSERT INTO app_messages (id, content, created_at, is_read)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            created_at = excluded.created_at,
            is_read = excluded.is_read",
    )
    .bind(&message.id)
    .bind(&message.content)
    .bind(message.created_at.to_rfc3339())
    .bind(if message.is_read { 1 } else { 0 })
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_read(pool: &SqlitePool, message_id: &str) -> Result<()> {
    sqlx::query("UPDATE app_messages SET is_read = 1 WHERE id = ?")
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, message_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM app_messages WHERE id = ?")
        .bind(message_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn clear(pool: &SqlitePool) -> Result<()> {
    sqlx::query("DELETE FROM app_messages")
        .execute(pool)
        .await?;
    Ok(())
}
