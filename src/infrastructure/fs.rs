use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct AppPaths {
    root: PathBuf,
    db: PathBuf,
    backgrounds: PathBuf,
    java: PathBuf,
    downloads: PathBuf,
    logs: PathBuf,
    cache: PathBuf,
    runtime: PathBuf,
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let base = dirs::data_local_dir()
            .or_else(dirs::home_dir)
            .context("failed to determine a writable local data directory")?;
        let root = base.join("ZeperionLauncher");

        Ok(Self {
            db: root.join("db").join("zeperion.sqlite"),
            backgrounds: root.join("backgrounds"),
            java: root.join("java"),
            downloads: root.join("downloads"),
            logs: root.join("logs"),
            cache: root.join("cache"),
            runtime: root.join("runtime"),
            root,
        })
    }

    pub async fn ensure(&self) -> Result<()> {
        let dirs = [
            self.root(),
            self.db_file().parent().expect("db parent exists"),
            self.backgrounds_dir(),
            self.java_dir(),
            self.downloads_dir(),
            self.logs_dir(),
            self.cache_dir(),
            self.runtime_dir(),
        ];

        for dir in dirs {
            tokio::fs::create_dir_all(dir).await?;
        }

        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn db_file(&self) -> &Path {
        &self.db
    }

    pub fn backgrounds_dir(&self) -> &Path {
        &self.backgrounds
    }

    pub fn java_dir(&self) -> &Path {
        &self.java
    }

    pub fn downloads_dir(&self) -> &Path {
        &self.downloads
    }

    pub fn logs_dir(&self) -> &Path {
        &self.logs
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache
    }

    pub fn runtime_dir(&self) -> &Path {
        &self.runtime
    }

    pub fn authlib_injector_jar(&self) -> PathBuf {
        self.runtime.join("authlib-injector.jar")
    }
}
