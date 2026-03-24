#![allow(dead_code)]

use anyhow::Result;
use chrono::Utc;
use sqlx::{Row, SqlitePool};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WindowPreferences {
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub is_maximized: bool,
}

impl Default for WindowPreferences {
    fn default() -> Self {
        Self {
            width: 970.0,
            height: 255.0,
            x: None,
            y: None,
            is_maximized: false,
        }
    }
}

pub async fn load(pool: &SqlitePool) -> Result<WindowPreferences> {
    let row = sqlx::query(
        "SELECT width, height, x, y, is_maximized
         FROM window_prefs
         WHERE id = 'main'
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(WindowPreferences::default());
    };

    let width = row.try_get::<f64, _>("width")?;
    let height = row.try_get::<f64, _>("height")?;
    let width = width.clamp(860.0, 1600.0);
    let height = height.clamp(255.0, 1200.0);
    let aspect_ratio = width / height;
    let is_legacy_default =
        (width - 1200.0).abs() < f64::EPSILON && (height - 760.0).abs() < f64::EPSILON;
    let is_previous_default =
        (width - 1080.0).abs() < f64::EPSILON && (height - 720.0).abs() < f64::EPSILON;
    let is_compact_default =
        (width - 980.0).abs() < f64::EPSILON && (height - 660.0).abs() < f64::EPSILON;
    let (width, height) = if is_legacy_default
        || is_previous_default
        || is_compact_default
        || !(1.15..=2.1).contains(&aspect_ratio)
    {
        (
            WindowPreferences::default().width,
            WindowPreferences::default().height,
        )
    } else {
        (width, height)
    };

    Ok(WindowPreferences {
        width,
        height,
        x: row.try_get::<Option<f64>, _>("x")?,
        y: row.try_get::<Option<f64>, _>("y")?,
        is_maximized: row.try_get::<i64, _>("is_maximized")? != 0,
    })
}

pub async fn upsert(pool: &SqlitePool, prefs: &WindowPreferences) -> Result<()> {
    sqlx::query(
        "INSERT INTO window_prefs (id, width, height, x, y, is_maximized, updated_at)
         VALUES ('main', ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            width = excluded.width,
            height = excluded.height,
            x = excluded.x,
            y = excluded.y,
            is_maximized = excluded.is_maximized,
            updated_at = excluded.updated_at",
    )
    .bind(prefs.width)
    .bind(prefs.height)
    .bind(prefs.x)
    .bind(prefs.y)
    .bind(i64::from(prefs.is_maximized as i32))
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}
