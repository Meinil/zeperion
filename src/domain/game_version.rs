use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InstalledGameVersion {
    pub id: String,
    pub version_name: String,
    pub version_type: String,
    pub release_time: DateTime<Utc>,
    pub install_dir: String,
    pub is_downloaded: bool,
    pub integrity_status: String,
}

impl InstalledGameVersion {
    pub fn is_launchable(&self) -> bool {
        self.is_downloaded && self.integrity_status == "complete"
    }
}
