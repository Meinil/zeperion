use anyhow::Result;
use reqwest::Url;

use crate::domain::{
    download::{RemoteGameVersion, RemoteVersionDetails, VersionManifestResponse},
    settings::{BMCLAPI_MIRROR_URL, LEGACY_BMCLAPI_DOC_URL, MirrorSource, MirrorType},
};

const OFFICIAL_BASE: &str = "https://piston-meta.mojang.com/";
const OFFICIAL_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest.json";
const OFFICIAL_ASSET_HOST: &str = "https://resources.download.minecraft.net/";
const OFFICIAL_LIBRARY_HOST: &str = "libraries.minecraft.net";
const FORGE_MAVEN_HOST: &str = "maven.minecraftforge.net";
const LEGACY_FORGE_MAVEN_HOST: &str = "files.minecraftforge.net";

#[derive(Clone, Debug)]
pub struct DownloadSourceProvider {
    base_url: String,
    mirror_type: MirrorType,
}

impl DownloadSourceProvider {
    pub fn new(base_url: impl Into<String>, mirror_type: MirrorType) -> Self {
        Self {
            base_url: normalize_base_url(&base_url.into()),
            mirror_type,
        }
    }

    pub fn version_manifest_url(&self) -> String {
        if self.uses_official_compatible_layout() {
            OFFICIAL_MANIFEST_URL.to_string()
        } else {
            join_url(&self.base_url, "mc/game/version_manifest.json")
        }
    }

    pub fn rewrite_metadata_url(&self, url: &str) -> String {
        self.rewrite_download_url(url)
    }

    pub fn rewrite_download_url(&self, url: &str) -> String {
        if self.uses_official_compatible_layout() {
            return url.to_string();
        }

        if let Some(rewritten) = self.rewrite_asset_url(url) {
            return rewritten;
        }

        if let Some(rewritten) = self.rewrite_library_url(url) {
            return rewritten;
        }

        if let Ok(parsed) = Url::parse(url) {
            let path = parsed.path().trim_start_matches('/');
            if path.is_empty() {
                self.base_url.clone()
            } else {
                join_url(&self.base_url, path)
            }
        } else {
            url.to_string()
        }
    }

    pub async fn fetch_version_manifest(&self) -> Result<Vec<RemoteGameVersion>> {
        let response = reqwest::get(self.version_manifest_url())
            .await?
            .error_for_status()?
            .json::<VersionManifestResponse>()
            .await?;

        Ok(response
            .versions
            .into_iter()
            .map(|mut version| {
                version.url = self.rewrite_metadata_url(&version.url);
                version
            })
            .collect())
    }

    pub async fn fetch_version_details(&self, url: &str) -> Result<RemoteVersionDetails> {
        let response = reqwest::get(self.rewrite_metadata_url(url))
            .await?
            .error_for_status()?
            .json::<RemoteVersionDetails>()
            .await?;

        Ok(response)
    }

    fn is_bmclapi(&self) -> bool {
        self.mirror_type == MirrorType::BmclApiCompatible
            || self.base_url == normalize_base_url(BMCLAPI_MIRROR_URL)
            || self.base_url == normalize_base_url(LEGACY_BMCLAPI_DOC_URL)
    }

    fn uses_official_compatible_layout(&self) -> bool {
        !self.is_bmclapi()
    }

    fn rewrite_asset_url(&self, url: &str) -> Option<String> {
        if self.uses_official_compatible_layout() {
            return None;
        }
        let parsed = Url::parse(url).ok()?;
        let official_assets = Url::parse(OFFICIAL_ASSET_HOST).ok()?;
        if parsed.domain() != official_assets.domain() {
            return None;
        }

        let path = parsed.path().trim_start_matches('/');
        let hash = path.split('/').nth(1)?;
        Some(join_url(&self.base_url, &format!("assets/{hash}")))
    }

    fn rewrite_library_url(&self, url: &str) -> Option<String> {
        if self.uses_official_compatible_layout() {
            return None;
        }
        let parsed = Url::parse(url).ok()?;
        let domain = parsed.domain()?;
        let path = parsed.path().trim_start_matches('/');

        let maven_path = if domain == OFFICIAL_LIBRARY_HOST || domain == FORGE_MAVEN_HOST {
            Some(path.to_string())
        } else if domain == LEGACY_FORGE_MAVEN_HOST {
            path.strip_prefix("maven/").map(str::to_string)
        } else {
            None
        }?;

        Some(join_url(&self.base_url, &format!("maven/{maven_path}")))
    }
}

pub fn provider_for_source(source: &MirrorSource) -> DownloadSourceProvider {
    DownloadSourceProvider::new(&source.base_url, source.mirror_type.clone())
}

fn join_url(base: &str, path: &str) -> String {
    format!(
        "{}{}",
        normalize_base_url(base),
        path.trim_start_matches('/')
    )
}

fn normalize_base_url(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        OFFICIAL_BASE.to_string()
    } else if trimmed.ends_with('/') {
        trimmed.to_string()
    } else {
        format!("{trimmed}/")
    }
}

#[cfg(test)]
mod tests {
    use super::{DownloadSourceProvider, OFFICIAL_BASE};
    use crate::domain::settings::{BMCLAPI_MIRROR_URL, LEGACY_BMCLAPI_DOC_URL, MirrorType};

    #[test]
    fn keeps_official_urls_unchanged() {
        let provider = DownloadSourceProvider::new(OFFICIAL_BASE, MirrorType::OfficialCompatible);
        let url = "https://libraries.minecraft.net/com/mojang/patchy/2.2.10/patchy-2.2.10.jar";
        assert_eq!(provider.rewrite_download_url(url), url);
    }

    #[test]
    fn rewrites_library_urls_to_bmcl_maven() {
        let provider =
            DownloadSourceProvider::new(BMCLAPI_MIRROR_URL, MirrorType::BmclApiCompatible);
        let url = "https://libraries.minecraft.net/com/mojang/patchy/2.2.10/patchy-2.2.10.jar";
        assert_eq!(
            provider.rewrite_download_url(url),
            "https://bmclapi2.bangbang93.com/maven/com/mojang/patchy/2.2.10/patchy-2.2.10.jar"
        );
    }

    #[test]
    fn rewrites_assets_to_bmcl_assets() {
        let provider =
            DownloadSourceProvider::new(BMCLAPI_MIRROR_URL, MirrorType::BmclApiCompatible);
        let url = "https://resources.download.minecraft.net/ab/abcdef1234567890";
        assert_eq!(
            provider.rewrite_download_url(url),
            "https://bmclapi2.bangbang93.com/assets/abcdef1234567890"
        );
    }

    #[test]
    fn preserves_version_package_path_when_rewriting_metadata() {
        let provider =
            DownloadSourceProvider::new(BMCLAPI_MIRROR_URL, MirrorType::BmclApiCompatible);
        let url = "https://piston-meta.mojang.com/v1/packages/bce21f96530d9b66a8bf81a4187e31e12cde099f/1.19.json";
        assert_eq!(
            provider.rewrite_metadata_url(url),
            "https://bmclapi2.bangbang93.com/v1/packages/bce21f96530d9b66a8bf81a4187e31e12cde099f/1.19.json"
        );
    }

    #[test]
    fn treats_legacy_bmcl_doc_host_as_bmclapi() {
        let provider =
            DownloadSourceProvider::new(LEGACY_BMCLAPI_DOC_URL, MirrorType::BmclApiCompatible);
        let url = "https://libraries.minecraft.net/com/mojang/patchy/2.2.10/patchy-2.2.10.jar";
        assert_eq!(
            provider.rewrite_download_url(url),
            "https://bmclapidoc.bangbang93.com/maven/com/mojang/patchy/2.2.10/patchy-2.2.10.jar"
        );
    }

    #[test]
    fn custom_mirror_defaults_to_official_compatible_mode() {
        let provider = DownloadSourceProvider::new(
            "https://example.com/mirror/",
            MirrorType::OfficialCompatible,
        );
        let library_url =
            "https://libraries.minecraft.net/com/mojang/patchy/2.2.10/patchy-2.2.10.jar";
        let asset_url = "https://resources.download.minecraft.net/ab/abcdef1234567890";
        assert_eq!(provider.rewrite_download_url(library_url), library_url);
        assert_eq!(provider.rewrite_download_url(asset_url), asset_url);
        assert_eq!(
            provider.version_manifest_url(),
            "https://piston-meta.mojang.com/mc/game/version_manifest.json"
        );
    }
}
