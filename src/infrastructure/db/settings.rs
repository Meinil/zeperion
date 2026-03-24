use anyhow::Result;
use sqlx::{Row, SqlitePool};

use crate::domain::settings::{AppSettings, normalize_selected_mirror_url};

pub async fn load(pool: &SqlitePool) -> Result<AppSettings> {
    let rows = sqlx::query("SELECT key, value FROM app_settings")
        .fetch_all(pool)
        .await?;

    if rows.is_empty() {
        let defaults = AppSettings::default();
        save(pool, &defaults).await?;
        return Ok(defaults);
    }

    let mut settings = AppSettings::default();
    for row in rows {
        let key: String = row.try_get("key")?;
        let value: String = row.try_get("value")?;
        match key.as_str() {
            "language" => settings.language = value,
            "theme_color" => settings.theme_color = value,
            "font_family" => settings.font_family = value,
            "font_size" => settings.font_size = value.parse().unwrap_or(settings.font_size),
            "game_memory_mb" => {
                settings.game_memory_mb = value.parse().unwrap_or(settings.game_memory_mb)
            }
            "resolution_width" => {
                settings.resolution_width = value.parse().unwrap_or(settings.resolution_width)
            }
            "resolution_height" => {
                settings.resolution_height = value.parse().unwrap_or(settings.resolution_height)
            }
            "selected_mirror" => settings.selected_mirror = normalize_selected_mirror_url(&value),
            "rotate_backgrounds" => settings.rotate_backgrounds = value == "true",
            "microsoft_client_id" => settings.microsoft_client_id = value,
            _ => {}
        }
    }

    Ok(settings)
}

pub async fn save(pool: &SqlitePool, settings: &AppSettings) -> Result<()> {
    let entries = [
        ("language", settings.language.clone()),
        ("theme_color", settings.theme_color.clone()),
        ("font_family", settings.font_family.clone()),
        ("font_size", settings.font_size.to_string()),
        ("game_memory_mb", settings.game_memory_mb.to_string()),
        ("resolution_width", settings.resolution_width.to_string()),
        ("resolution_height", settings.resolution_height.to_string()),
        (
            "selected_mirror",
            normalize_selected_mirror_url(&settings.selected_mirror),
        ),
        (
            "rotate_backgrounds",
            settings.rotate_backgrounds.to_string(),
        ),
        ("microsoft_client_id", settings.microsoft_client_id.clone()),
    ];

    for (key, value) in entries {
        sqlx::query(
            "INSERT INTO app_settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    }

    Ok(())
}
