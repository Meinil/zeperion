use anyhow::Result;

use crate::domain::{
    download::{RemoteGameVersion, RemoteVersionDetails},
    settings::MirrorSource,
};

use super::provider::provider_for_source;

pub async fn fetch_version_manifest_with_source(
    source: &MirrorSource,
) -> Result<Vec<RemoteGameVersion>> {
    provider_for_source(source).fetch_version_manifest().await
}

pub async fn fetch_version_details_with_source(
    source: &MirrorSource,
    url: &str,
) -> Result<RemoteVersionDetails> {
    provider_for_source(source).fetch_version_details(url).await
}
