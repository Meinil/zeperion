use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum JavaSource {
    AutoScan,
    ImportedFolder,
    ImportedArchive,
}

impl JavaSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AutoScan => "auto_scan",
            Self::ImportedFolder => "imported_folder",
            Self::ImportedArchive => "imported_archive",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "imported_folder" => Self::ImportedFolder,
            "imported_archive" => Self::ImportedArchive,
            _ => Self::AutoScan,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JavaRuntime {
    pub id: String,
    pub name: String,
    pub version: String,
    pub path: String,
    pub source: JavaSource,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
}

impl JavaRuntime {
    pub fn scanned(name: String, version: String, path: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            version,
            path,
            source: JavaSource::AutoScan,
            is_default: false,
            created_at: Utc::now(),
        }
    }
}
