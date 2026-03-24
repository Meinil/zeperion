#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "running" => Self::Running,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Pending,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Paused | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DownloadTaskMetadata {
    pub kind: String,
    #[serde(default)]
    pub detail_url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub resume_offset: Option<i64>,
    #[serde(default)]
    pub part_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DownloadTaskRecord {
    pub id: String,
    pub package_id: String,
    pub package_name: String,
    pub task_type: String,
    pub status: DownloadStatus,
    pub downloaded_bytes: i64,
    pub total_bytes: Option<i64>,
    pub target_path: String,
    pub metadata_json: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DownloadCheckpoint {
    pub downloaded_bytes: i64,
    pub total_bytes: Option<i64>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub part_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VersionManifestResponse {
    pub latest: ManifestLatest,
    pub versions: Vec<RemoteGameVersion>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ManifestLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RemoteGameVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RemoteVersionDetails {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    #[serde(rename = "mainClass", default)]
    pub main_class: Option<String>,
    pub assets: Option<String>,
    pub downloads: VersionDownloads,
    #[serde(rename = "assetIndex", default)]
    pub asset_index: Option<AssetIndexInfo>,
    #[serde(default)]
    pub libraries: Vec<RemoteLibrary>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VersionDownloads {
    pub client: DownloadArtifact,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DownloadArtifact {
    pub sha1: String,
    pub size: i64,
    pub url: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AssetIndexInfo {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AssetIndexFile {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AssetObject {
    pub hash: String,
    pub size: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RemoteLibrary {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<DownloadArtifact>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DownloadFilters {
    pub show_release: bool,
    pub show_snapshot: bool,
    pub show_old: bool,
    pub show_downloaded_only: bool,
}

impl Default for DownloadFilters {
    fn default() -> Self {
        Self {
            show_release: true,
            show_snapshot: true,
            show_old: true,
            show_downloaded_only: false,
        }
    }
}

impl RemoteGameVersion {
    pub fn matches_filters(&self, filters: &DownloadFilters, installed: bool) -> bool {
        let type_match = match self.version_type.as_str() {
            "release" => filters.show_release,
            "snapshot" => filters.show_snapshot,
            _ => filters.show_old,
        };

        type_match && (!filters.show_downloaded_only || installed)
    }
}

impl RemoteLibrary {
    pub fn artifact_for_current_platform(&self) -> Option<&DownloadArtifact> {
        let artifact = self.downloads.as_ref()?.artifact.as_ref()?;
        if library_artifact_matches_current_platform(&self.name) {
            Some(artifact)
        } else {
            None
        }
    }
}

fn library_artifact_matches_current_platform(name: &str) -> bool {
    let Some(classifier) = name.split(':').nth(3) else {
        return true;
    };

    if !classifier.starts_with("natives-") {
        return true;
    }

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match os {
        "macos" => match arch {
            "aarch64" => matches!(classifier, "natives-macos-arm64" | "natives-macos-patch"),
            _ => matches!(classifier, "natives-macos" | "natives-macos-patch"),
        },
        "windows" => match arch {
            "aarch64" => classifier == "natives-windows-arm64",
            "x86" => classifier == "natives-windows-x86",
            _ => classifier == "natives-windows",
        },
        "linux" => classifier == "natives-linux",
        _ => false,
    }
}

impl DownloadTaskRecord {
    pub fn metadata(&self) -> DownloadTaskMetadata {
        serde_json::from_str(&self.metadata_json).unwrap_or_default()
    }

    pub fn with_status(mut self, status: DownloadStatus) -> Self {
        self.status = status;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_progress(mut self, downloaded_bytes: i64, total_bytes: Option<i64>) -> Self {
        self.downloaded_bytes = downloaded_bytes;
        self.total_bytes = total_bytes;
        self.updated_at = Utc::now();
        self
    }

    pub fn progress_ratio(&self) -> Option<f32> {
        self.total_bytes.map(|total| {
            if total <= 0 {
                0.0
            } else {
                (self.downloaded_bytes as f32 / total as f32).clamp(0.0, 1.0)
            }
        })
    }

    pub fn is_resumable(&self) -> bool {
        self.status.is_retryable()
    }

    pub fn is_completed(&self) -> bool {
        self.status == DownloadStatus::Completed
    }

    pub fn checkpoint(&self) -> DownloadCheckpoint {
        let metadata = self.metadata();
        DownloadCheckpoint {
            downloaded_bytes: self.downloaded_bytes,
            total_bytes: self.total_bytes,
            source_url: metadata.source_url,
            sha1: metadata.sha1,
            part_path: metadata.part_path,
        }
    }

    pub fn metadata_sync(
        package_id: String,
        package_name: String,
        target_path: String,
        detail_url: Option<String>,
    ) -> Self {
        let metadata_json = serde_json::to_string(&DownloadTaskMetadata {
            kind: "metadata_sync".to_string(),
            detail_url,
            source_url: None,
            sha1: None,
            resume_offset: None,
            part_path: None,
        })
        .unwrap_or_else(|_| "{}".to_string());
        Self {
            id: Uuid::new_v4().to_string(),
            package_id,
            package_name,
            task_type: "metadata_sync".to_string(),
            status: DownloadStatus::Completed,
            downloaded_bytes: 0,
            total_bytes: None,
            target_path,
            metadata_json,
            updated_at: Utc::now(),
        }
    }

    pub fn core_download(
        package_id: String,
        package_name: String,
        target_path: String,
        total_bytes: Option<i64>,
        downloaded_bytes: i64,
        status: DownloadStatus,
        detail_url: Option<String>,
    ) -> Self {
        let metadata_json = serde_json::to_string(&DownloadTaskMetadata {
            kind: "core_download".to_string(),
            detail_url,
            source_url: None,
            sha1: None,
            resume_offset: None,
            part_path: None,
        })
        .unwrap_or_else(|_| "{}".to_string());
        Self {
            id: format!("core-{package_id}"),
            package_id,
            package_name,
            task_type: "core_download".to_string(),
            status,
            downloaded_bytes,
            total_bytes,
            target_path,
            metadata_json,
            updated_at: Utc::now(),
        }
    }

    pub fn client_download(
        package_id: String,
        package_name: String,
        target_path: String,
        total_bytes: i64,
        status: DownloadStatus,
    ) -> Self {
        let downloaded_bytes = if matches!(status, DownloadStatus::Completed) {
            total_bytes
        } else {
            0
        };
        Self {
            id: format!("client-{package_id}"),
            package_id,
            package_name,
            task_type: "client_download".to_string(),
            status,
            downloaded_bytes,
            total_bytes: Some(total_bytes),
            target_path,
            metadata_json: serde_json::to_string(&DownloadTaskMetadata {
                kind: "client_download".to_string(),
                detail_url: None,
                source_url: None,
                sha1: None,
                resume_offset: None,
                part_path: None,
            })
            .unwrap_or_else(|_| "{}".to_string()),
            updated_at: Utc::now(),
        }
    }

    pub fn libraries_download(
        package_id: String,
        package_name: String,
        target_path: String,
        total_bytes: i64,
        status: DownloadStatus,
    ) -> Self {
        let downloaded_bytes = if matches!(status, DownloadStatus::Completed) {
            total_bytes
        } else {
            0
        };
        Self {
            id: format!("libraries-{package_id}"),
            package_id,
            package_name,
            task_type: "libraries_download".to_string(),
            status,
            downloaded_bytes,
            total_bytes: Some(total_bytes),
            target_path,
            metadata_json: serde_json::to_string(&DownloadTaskMetadata {
                kind: "libraries_download".to_string(),
                detail_url: None,
                source_url: None,
                sha1: None,
                resume_offset: None,
                part_path: None,
            })
            .unwrap_or_else(|_| "{}".to_string()),
            updated_at: Utc::now(),
        }
    }

    pub fn assets_sync(
        package_id: String,
        package_name: String,
        target_path: String,
        total_bytes: i64,
        status: DownloadStatus,
    ) -> Self {
        let downloaded_bytes = if matches!(status, DownloadStatus::Completed) {
            total_bytes
        } else {
            0
        };
        Self {
            id: format!("assets-{package_id}"),
            package_id,
            package_name,
            task_type: "assets_sync".to_string(),
            status,
            downloaded_bytes,
            total_bytes: Some(total_bytes),
            target_path,
            metadata_json: serde_json::to_string(&DownloadTaskMetadata {
                kind: "assets_sync".to_string(),
                detail_url: None,
                source_url: None,
                sha1: None,
                resume_offset: None,
                part_path: None,
            })
            .unwrap_or_else(|_| "{}".to_string()),
            updated_at: Utc::now(),
        }
    }

    pub fn asset_objects_download(
        package_id: String,
        package_name: String,
        target_path: String,
        total_bytes: i64,
        status: DownloadStatus,
    ) -> Self {
        let downloaded_bytes = if matches!(status, DownloadStatus::Completed) {
            total_bytes
        } else {
            0
        };
        Self {
            id: format!("asset-objects-{package_id}"),
            package_id,
            package_name,
            task_type: "asset_objects_download".to_string(),
            status,
            downloaded_bytes,
            total_bytes: Some(total_bytes),
            target_path,
            metadata_json: serde_json::to_string(&DownloadTaskMetadata {
                kind: "asset_objects_download".to_string(),
                detail_url: None,
                source_url: None,
                sha1: None,
                resume_offset: None,
                part_path: None,
            })
            .unwrap_or_else(|_| "{}".to_string()),
            updated_at: Utc::now(),
        }
    }

    pub fn with_metadata(mut self, metadata: DownloadTaskMetadata) -> Self {
        self.metadata_json = serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{DownloadArtifact, LibraryDownloads, RemoteLibrary};

    fn library(name: &str, path: &str) -> RemoteLibrary {
        RemoteLibrary {
            name: name.to_string(),
            downloads: Some(LibraryDownloads {
                artifact: Some(DownloadArtifact {
                    sha1: "dummy".to_string(),
                    size: 1,
                    url: "https://example.com/lib.jar".to_string(),
                    path: path.to_string(),
                }),
            }),
        }
    }

    #[test]
    fn keeps_regular_library_artifacts() {
        let library = library(
            "org.lwjgl:lwjgl-opengl:3.3.3",
            "org/lwjgl/lwjgl-opengl/3.3.3/lwjgl-opengl-3.3.3.jar",
        );
        assert!(library.artifact_for_current_platform().is_some());
    }

    #[test]
    fn filters_non_matching_native_artifacts_for_current_platform() {
        let windows = library(
            "org.lwjgl:lwjgl-openal:3.3.3:natives-windows",
            "org/lwjgl/lwjgl-openal/3.3.3/lwjgl-openal-3.3.3-natives-windows.jar",
        );
        let macos = library(
            "org.lwjgl:lwjgl-openal:3.3.3:natives-macos",
            "org/lwjgl/lwjgl-openal/3.3.3/lwjgl-openal-3.3.3-natives-macos.jar",
        );
        let macos_arm64 = library(
            "org.lwjgl:lwjgl-openal:3.3.3:natives-macos-arm64",
            "org/lwjgl/lwjgl-openal/3.3.3/lwjgl-openal-3.3.3-natives-macos-arm64.jar",
        );

        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("macos", "aarch64") => {
                assert!(windows.artifact_for_current_platform().is_none());
                assert!(macos.artifact_for_current_platform().is_none());
                assert!(macos_arm64.artifact_for_current_platform().is_some());
            }
            ("macos", _) => {
                assert!(windows.artifact_for_current_platform().is_none());
                assert!(macos.artifact_for_current_platform().is_some());
                assert!(macos_arm64.artifact_for_current_platform().is_none());
            }
            ("windows", arch) => {
                assert!(macos.artifact_for_current_platform().is_none());
                assert!(macos_arm64.artifact_for_current_platform().is_none());
                if arch == "aarch64" {
                    assert!(windows.artifact_for_current_platform().is_none());
                } else {
                    assert!(windows.artifact_for_current_platform().is_some());
                }
            }
            ("linux", _) => {
                assert!(windows.artifact_for_current_platform().is_none());
                assert!(macos.artifact_for_current_platform().is_none());
                assert!(macos_arm64.artifact_for_current_platform().is_none());
            }
            _ => {}
        }
    }
}
