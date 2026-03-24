use anyhow::Result;
use sqlx::{Row, SqlitePool};

use crate::domain::settings::{
    BMCLAPI_MIRROR_URL, LEGACY_BMCLAPI_DOC_URL, MirrorSource, MirrorType, OFFICIAL_MIRROR_URL,
};

const OFFICIAL_MIRROR_ID: &str = "builtin-official";
const OFFICIAL_MIRROR_NAME: &str = "Official";
const OFFICIAL_MIRROR_TYPE: &str = "official_compatible";
const BMCLAPI_MIRROR_ID: &str = "builtin-bmclapi";
const BMCLAPI_MIRROR_NAME: &str = "BMCLAPI";
const BMCLAPI_MIRROR_TYPE: &str = "bmclapi_compatible";

pub async fn load_all(pool: &SqlitePool) -> Result<Vec<MirrorSource>> {
    let rows = sqlx::query(
        "SELECT id, name, base_url, is_builtin, mirror_type
         FROM custom_mirrors
         ORDER BY is_builtin DESC, name ASC",
    )
    .fetch_all(pool)
    .await?;

    let mut mirrors = Vec::with_capacity(rows.len());
    for row in rows {
        mirrors.push(MirrorSource {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            base_url: row.try_get("base_url")?,
            is_builtin: row.try_get::<i64, _>("is_builtin")? == 1,
            mirror_type: MirrorType::from_str(&row.try_get::<String, _>("mirror_type")?),
        });
    }

    Ok(mirrors)
}

pub async fn insert(pool: &SqlitePool, mirror: &MirrorSource) -> Result<()> {
    sqlx::query(
        "INSERT INTO custom_mirrors (id, name, base_url, is_builtin, mirror_type)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&mirror.id)
    .bind(&mirror.name)
    .bind(&mirror.base_url)
    .bind(if mirror.is_builtin { 1 } else { 0 })
    .bind(mirror.mirror_type.as_str())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, mirror_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM custom_mirrors WHERE id = ? AND is_builtin = 0")
        .bind(mirror_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn ensure_builtin(pool: &SqlitePool) -> Result<()> {
    ensure_schema(pool).await?;

    for (id, name, url, mirror_type) in [
        (
            OFFICIAL_MIRROR_ID,
            OFFICIAL_MIRROR_NAME,
            OFFICIAL_MIRROR_URL,
            OFFICIAL_MIRROR_TYPE,
        ),
        (
            BMCLAPI_MIRROR_ID,
            BMCLAPI_MIRROR_NAME,
            BMCLAPI_MIRROR_URL,
            BMCLAPI_MIRROR_TYPE,
        ),
    ] {
        sqlx::query(
            "INSERT OR IGNORE INTO custom_mirrors (id, name, base_url, is_builtin, mirror_type)
             VALUES (?, ?, ?, 1, ?)",
        )
        .bind(id)
        .bind(name)
        .bind(url)
        .bind(mirror_type)
        .execute(pool)
        .await?;

        sqlx::query("UPDATE custom_mirrors SET mirror_type = ? WHERE id = ?")
            .bind(mirror_type)
            .bind(id)
            .execute(pool)
            .await?;

        sqlx::query("UPDATE custom_mirrors SET base_url = ? WHERE id = ?")
            .bind(url)
            .bind(id)
            .execute(pool)
            .await?;
    }

    sqlx::query("UPDATE custom_mirrors SET base_url = ? WHERE base_url = ?")
        .bind(BMCLAPI_MIRROR_URL)
        .bind(LEGACY_BMCLAPI_DOC_URL)
        .execute(pool)
        .await?;

    Ok(())
}

async fn ensure_schema(pool: &SqlitePool) -> Result<()> {
    let _ = sqlx::query(
        "ALTER TABLE custom_mirrors ADD COLUMN mirror_type TEXT NOT NULL DEFAULT 'official_compatible'",
    )
    .execute(pool)
    .await;

    Ok(())
}
