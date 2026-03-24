use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BackgroundSource {
    LocalFile,
    Url,
}

impl BackgroundSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalFile => "local_file",
            Self::Url => "url",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BackgroundImage {
    pub id: String,
    pub source_type: BackgroundSource,
    pub source_value: String,
    pub local_path: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
