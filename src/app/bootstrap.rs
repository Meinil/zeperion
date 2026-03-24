use std::sync::{Arc, OnceLock};

use anyhow::Result;

use crate::{
    domain::{
        account::Account, background::BackgroundImage, download::DownloadTaskRecord,
        game_version::InstalledGameVersion, settings::MirrorSource,
    },
    infrastructure::{db, fs::AppPaths, i18n::I18n},
    platform,
    state::AppStore,
};

static BOOTSTRAP: OnceLock<BootstrapData> = OnceLock::new();

#[derive(Clone)]
pub struct AppServices {
    pub paths: Arc<AppPaths>,
    pub pool: sqlx::SqlitePool,
    pub i18n: Arc<I18n>,
}

#[derive(Clone)]
pub struct BootstrapData {
    pub services: Arc<AppServices>,
    pub initial_store: AppStore,
    pub window_prefs: db::window_prefs::WindowPreferences,
}

pub async fn bootstrap() -> Result<BootstrapData> {
    let paths = Arc::new(AppPaths::discover()?);
    paths.ensure().await?;

    let pool = db::connect(paths.db_file()).await?;
    db::migrate(&pool).await?;
    db::custom_mirrors::ensure_builtin(&pool).await?;

    let i18n = Arc::new(I18n::load("zh-CN")?);
    let mut settings = db::settings::load(&pool).await?;
    let normalized_mirror =
        crate::domain::settings::normalize_selected_mirror_url(&settings.selected_mirror);
    if settings.selected_mirror != normalized_mirror {
        settings.selected_mirror = normalized_mirror;
        db::settings::save(&pool, &settings).await?;
    }

    let mut java_runtimes = db::java::load_all(&pool).await?;
    if java_runtimes.is_empty() {
        java_runtimes = platform::java::scan_java_runtimes().await;
        db::java::replace_all(&pool, &java_runtimes).await?;
    }

    if java_runtimes.iter().all(|runtime| !runtime.is_default) {
        if let Some(first) = java_runtimes.first_mut() {
            first.is_default = true;
            db::java::set_default(&pool, &first.id).await?;
        }
    }

    let mut accounts: Vec<Account> = db::accounts::load_all(&pool).await?;
    if accounts.iter().all(|account| !account.selected) {
        if let Some(first) = accounts.first_mut() {
            first.selected = true;
            db::accounts::set_selected(&pool, &first.id).await?;
        }
    }
    let versions: Vec<InstalledGameVersion> = db::versions::load_all(&pool).await?;
    let backgrounds: Vec<BackgroundImage> = db::backgrounds::load_all(&pool).await?;
    let mirrors: Vec<MirrorSource> = db::custom_mirrors::load_all(&pool).await?;
    let messages = db::messages::load_all(&pool).await?;
    let window_prefs = db::window_prefs::load(&pool).await?;
    let mut download_tasks: Vec<DownloadTaskRecord> = db::download_tasks::load_all(&pool).await?;
    for task in &mut download_tasks {
        if matches!(
            task.status,
            crate::domain::download::DownloadStatus::Running
        ) {
            task.status = crate::domain::download::DownloadStatus::Paused;
            task.updated_at = chrono::Utc::now();
            db::download_tasks::upsert(&pool, task).await?;
        }
    }

    let services = Arc::new(AppServices { paths, pool, i18n });

    let initial_store = AppStore::from_bootstrap(
        settings,
        accounts,
        java_runtimes,
        versions,
        backgrounds,
        mirrors,
        download_tasks,
        messages,
    );

    Ok(BootstrapData {
        services,
        initial_store,
        window_prefs,
    })
}

pub fn set_bootstrap(data: BootstrapData) {
    let _ = BOOTSTRAP.set(data);
}

pub fn get_bootstrap() -> BootstrapData {
    BOOTSTRAP
        .get()
        .expect("bootstrap has not been initialized")
        .clone()
}
