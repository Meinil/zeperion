use std::path::Path;

use anyhow::{Result, anyhow};

use crate::{
    domain::{
        download::{AssetIndexFile, DownloadArtifact, RemoteVersionDetails},
        game_version::InstalledGameVersion,
    },
    infrastructure::network::downloader,
};

pub async fn check_version_integrity(version: &InstalledGameVersion) -> Result<String> {
    let manifest_path = Path::new(&version.install_dir).join("manifest.json");
    if !manifest_path.exists() {
        return Err(anyhow!("manifest.json is missing"));
    }

    let manifest_bytes = tokio::fs::read(&manifest_path).await?;
    let details: RemoteVersionDetails = serde_json::from_slice(&manifest_bytes)?;

    let client_path = Path::new(&version.install_dir).join("client.jar");
    if !client_path.exists() {
        return Ok("metadata_only".to_string());
    }

    let client_bytes = tokio::fs::read(&client_path).await?;
    downloader::verify_sha1(&client_bytes, &details.downloads.client.sha1)?;

    let libraries_dir = Path::new(&version.install_dir).join("libraries");
    let library_artifacts: Vec<&DownloadArtifact> = details
        .libraries
        .iter()
        .filter_map(|library| library.artifact_for_current_platform())
        .collect();

    if library_artifacts.is_empty() {
        return Ok("client_only".to_string());
    }

    for artifact in library_artifacts {
        let library_path = libraries_dir.join(&artifact.path);
        if !library_path.exists() {
            return Ok("client_only".to_string());
        }
        let library_bytes = tokio::fs::read(&library_path).await?;
        downloader::verify_sha1(&library_bytes, &artifact.sha1)?;
    }

    let Some(asset_index) = details.asset_index.as_ref() else {
        return Ok("libraries_only".to_string());
    };

    let assets_root = Path::new(&version.install_dir).join("assets");
    let index_path = assets_root
        .join("indexes")
        .join(format!("{}.json", asset_index.id));
    if !index_path.exists() {
        return Ok("libraries_only".to_string());
    }

    let index_bytes = tokio::fs::read(&index_path).await?;
    downloader::verify_sha1(&index_bytes, &asset_index.sha1)?;
    let asset_index_file: AssetIndexFile = serde_json::from_slice(&index_bytes)?;

    for object in asset_index_file.objects.values() {
        let prefix = &object.hash[..2];
        let object_path = assets_root.join("objects").join(prefix).join(&object.hash);
        if !object_path.exists() {
            return Ok("libraries_only".to_string());
        }
        let object_bytes = tokio::fs::read(&object_path).await?;
        downloader::verify_sha1(&object_bytes, &object.hash)?;
    }

    Ok("complete".to_string())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::Utc;

    use super::check_version_integrity;
    use crate::domain::game_version::InstalledGameVersion;

    #[tokio::test]
    async fn returns_metadata_only_when_client_missing() {
        let dir = PathBuf::from("/tmp/zeperion-integrity-metadata-only");
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir)
            .await
            .expect("create test dir");
        tokio::fs::write(
            dir.join("manifest.json"),
            r#"{
              "id":"1.20.1",
              "type":"release",
              "releaseTime":"2024-01-01T00:00:00Z",
              "assets":"legacy",
              "downloads":{"client":{"sha1":"a94a8fe5ccb19ba61c4c0873d391e987982fbbd3","size":5,"url":"https://example.com/client.jar"}}
            }"#,
        )
        .await
        .expect("write manifest");

        let version = InstalledGameVersion {
            id: "v1".into(),
            version_name: "1.20.1".into(),
            version_type: "release".into(),
            release_time: Utc::now(),
            install_dir: dir.display().to_string(),
            is_downloaded: true,
            integrity_status: "metadata_only".into(),
        };

        let status = check_version_integrity(&version)
            .await
            .expect("integrity status");
        assert_eq!(status, "metadata_only");
    }
}
