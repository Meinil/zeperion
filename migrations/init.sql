-- Zeperion Launcher full initialization SQL
-- This file is a consolidated reference of the current schema and seed data.
-- It is intended for fresh initialization / inspection only.
-- Do not replace the incremental sqlx migration chain with this file directly.

BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS java_runtimes (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    source TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS accounts (
    id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    uuid TEXT NOT NULL,
    login_type TEXT NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    client_token TEXT,
    auth_server_url TEXT,
    authlib_metadata_b64 TEXT,
    oauth_flow TEXT,
    skin_mode TEXT,
    skin_reference TEXT,
    avatar_url TEXT,
    selected INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_validated_at TEXT
);

CREATE TABLE IF NOT EXISTS game_versions (
    id TEXT PRIMARY KEY NOT NULL,
    version_name TEXT NOT NULL UNIQUE,
    version_type TEXT NOT NULL,
    release_time TEXT NOT NULL,
    install_dir TEXT NOT NULL,
    is_downloaded INTEGER NOT NULL DEFAULT 0,
    integrity_status TEXT NOT NULL DEFAULT 'unknown'
);

CREATE TABLE IF NOT EXISTS background_images (
    id TEXT PRIMARY KEY NOT NULL,
    source_type TEXT NOT NULL,
    source_value TEXT NOT NULL,
    local_path TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS download_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    package_id TEXT NOT NULL,
    package_name TEXT NOT NULL,
    task_type TEXT NOT NULL,
    status TEXT NOT NULL,
    downloaded_bytes INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER,
    target_path TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS custom_mirrors (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL UNIQUE,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    mirror_type TEXT NOT NULL DEFAULT 'official_compatible'
);

CREATE TABLE IF NOT EXISTS launch_history (
    id TEXT PRIMARY KEY NOT NULL,
    version_name TEXT NOT NULL,
    account_name TEXT NOT NULL,
    java_path TEXT NOT NULL,
    command_line TEXT NOT NULL,
    log_path TEXT NOT NULL,
    status TEXT NOT NULL,
    exit_code INTEGER,
    error_message TEXT,
    launched_at TEXT NOT NULL,
    finished_at TEXT
);

CREATE TABLE IF NOT EXISTS window_prefs (
    id TEXT PRIMARY KEY NOT NULL,
    width REAL NOT NULL,
    height REAL NOT NULL,
    x REAL,
    y REAL,
    is_maximized INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_messages (
    id TEXT PRIMARY KEY NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    is_read INTEGER NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO custom_mirrors (id, name, base_url, is_builtin, mirror_type)
VALUES ('builtin-bmclapi', 'BMCLAPI', 'https://bmclapi2.bangbang93.com/', 1, 'bmclapi_compatible');

INSERT OR IGNORE INTO custom_mirrors (id, name, base_url, is_builtin, mirror_type)
VALUES ('builtin-official', 'Official', 'https://piston-meta.mojang.com/', 1, 'official_compatible');

COMMIT;
