use std::{
    future::Future,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use anyhow::{Result, anyhow};
use reqwest::{Client, StatusCode, header::RANGE};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    time::sleep,
};

const MAX_DOWNLOAD_ATTEMPTS: usize = 4;
const RETRYABLE_STATUSES: [StatusCode; 6] = [
    StatusCode::REQUEST_TIMEOUT,
    StatusCode::TOO_MANY_REQUESTS,
    StatusCode::BAD_GATEWAY,
    StatusCode::SERVICE_UNAVAILABLE,
    StatusCode::GATEWAY_TIMEOUT,
    StatusCode::INTERNAL_SERVER_ERROR,
];

pub async fn download_to_file_atomic_with_resume<F, Fut>(
    url: &str,
    target: &Path,
    control: Option<&Arc<AtomicU8>>,
    mut on_progress: F,
) -> Result<Option<()>>
where
    F: FnMut(i64, Option<i64>) -> Fut,
    Fut: Future<Output = ()>,
{
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).await?;
    }

    let part_path = part_path(target);
    let meta_path = meta_path(target);
    let client = Client::builder().build()?;

    for attempt in 1..=MAX_DOWNLOAD_ATTEMPTS {
        let mut existing = fs::metadata(&part_path)
            .await
            .map(|meta| meta.len() as i64)
            .unwrap_or(0);
        let existing_meta = read_resume_metadata(&meta_path).await.ok();

        if let Some(meta) = existing_meta.as_ref() {
            if meta.source_url != url {
                let _ = fs::remove_file(&part_path).await;
                let _ = fs::remove_file(&meta_path).await;
                existing = 0;
            } else if let Some(expected) = meta.expected_sha1.as_deref() {
                if expected.is_empty() {
                    let _ = fs::remove_file(&part_path).await;
                    let _ = fs::remove_file(&meta_path).await;
                    existing = 0;
                }
            }
        }

        let mut request = client.get(url);
        if existing > 0 {
            request = request.header(RANGE, format!("bytes={existing}-"));
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) if should_retry_request_error(&error) && attempt < MAX_DOWNLOAD_ATTEMPTS => {
                sleep(retry_delay(attempt)).await;
                continue;
            }
            Err(error) => return Err(error.into()),
        };

        if response.status() == StatusCode::RANGE_NOT_SATISFIABLE && existing > 0 {
            let _ = fs::remove_file(&part_path).await;
            let _ = fs::remove_file(&meta_path).await;
            if attempt < MAX_DOWNLOAD_ATTEMPTS {
                sleep(retry_delay(attempt)).await;
                continue;
            }
        }

        if RETRYABLE_STATUSES.contains(&response.status()) && attempt < MAX_DOWNLOAD_ATTEMPTS {
            sleep(retry_delay(attempt)).await;
            continue;
        }

        let mut response = response.error_for_status()?;
        let is_partial = response.status() == StatusCode::PARTIAL_CONTENT && existing > 0;
        let total_bytes = response
            .content_length()
            .map(|value| value as i64 + if is_partial { existing } else { 0 });

        let mut downloaded = if is_partial { existing } else { 0 };
        write_resume_metadata(
            &meta_path,
            &DownloadResumeMetadata {
                source_url: url.to_string(),
                expected_sha1: None,
                downloaded_bytes: downloaded,
                total_bytes,
            },
        )
        .await?;
        let mut resume_write_state = ResumeWriteState::new(downloaded);
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(is_partial)
            .truncate(!is_partial)
            .open(&part_path)
            .await?;

        on_progress(downloaded, total_bytes).await;

        loop {
            if is_interrupted(control) {
                file.flush().await?;
                write_resume_metadata(
                    &meta_path,
                    &DownloadResumeMetadata {
                        source_url: url.to_string(),
                        expected_sha1: None,
                        downloaded_bytes: downloaded,
                        total_bytes,
                    },
                )
                .await?;
                return Ok(None);
            }

            let next_chunk = match response.chunk().await {
                Ok(chunk) => chunk,
                Err(error)
                    if should_retry_request_error(&error) && attempt < MAX_DOWNLOAD_ATTEMPTS =>
                {
                    file.flush().await?;
                    write_resume_metadata(
                        &meta_path,
                        &DownloadResumeMetadata {
                            source_url: url.to_string(),
                            expected_sha1: None,
                            downloaded_bytes: downloaded,
                            total_bytes,
                        },
                    )
                    .await?;
                    sleep(retry_delay(attempt)).await;
                    break;
                }
                Err(error) => return Err(error.into()),
            };

            let Some(chunk) = next_chunk else {
                file.flush().await?;
                drop(file);
                fs::rename(&part_path, target).await?;
                let _ = fs::remove_file(&meta_path).await;
                return Ok(Some(()));
            };

            file.write_all(&chunk).await?;
            downloaded += chunk.len() as i64;
            if resume_write_state.should_flush(downloaded) {
                write_resume_metadata(
                    &meta_path,
                    &DownloadResumeMetadata {
                        source_url: url.to_string(),
                        expected_sha1: None,
                        downloaded_bytes: downloaded,
                        total_bytes,
                    },
                )
                .await?;
                resume_write_state.mark_flushed(downloaded);
            }
            on_progress(downloaded, total_bytes).await;
        }
    }

    Err(anyhow!("download failed after multiple retry attempts"))
}

pub async fn verify_file_sha1(target: &Path, expected_sha1: &str) -> Result<()> {
    let bytes = fs::read(target).await?;
    verify_sha1(&bytes, expected_sha1)
}

pub fn verify_sha1(bytes: &[u8], expected_sha1: &str) -> Result<()> {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());
    if actual == expected_sha1 {
        Ok(())
    } else {
        Err(anyhow!(
            "sha1 mismatch: expected {}, got {}",
            expected_sha1,
            actual
        ))
    }
}

fn part_path(target: &Path) -> PathBuf {
    let mut file_name = target
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "download.bin".to_string());
    file_name.push_str(".part");
    target.with_file_name(file_name)
}

fn meta_path(target: &Path) -> PathBuf {
    let mut file_name = target
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "download.bin".to_string());
    file_name.push_str(".part.meta");
    target.with_file_name(file_name)
}

fn is_interrupted(control: Option<&Arc<AtomicU8>>) -> bool {
    control
        .map(|value| value.load(Ordering::SeqCst) != 0)
        .unwrap_or(false)
}

fn should_retry_request_error(error: &reqwest::Error) -> bool {
    if error.is_timeout() || error.is_connect() || error.is_request() || error.is_body() {
        return true;
    }

    error
        .status()
        .map(|status| RETRYABLE_STATUSES.contains(&status))
        .unwrap_or(false)
}

fn retry_delay(attempt: usize) -> Duration {
    Duration::from_millis((attempt as u64).saturating_mul(500))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DownloadResumeMetadata {
    source_url: String,
    expected_sha1: Option<String>,
    downloaded_bytes: i64,
    total_bytes: Option<i64>,
}

#[derive(Debug)]
struct ResumeWriteState {
    last_written_bytes: i64,
    last_written_at: std::time::Instant,
}

impl ResumeWriteState {
    fn new(downloaded_bytes: i64) -> Self {
        Self {
            last_written_bytes: downloaded_bytes,
            last_written_at: std::time::Instant::now(),
        }
    }

    fn should_flush(&self, downloaded_bytes: i64) -> bool {
        let delta = downloaded_bytes.saturating_sub(self.last_written_bytes);
        delta >= 256 * 1024 || self.last_written_at.elapsed() >= Duration::from_millis(500)
    }

    fn mark_flushed(&mut self, downloaded_bytes: i64) {
        self.last_written_bytes = downloaded_bytes;
        self.last_written_at = std::time::Instant::now();
    }
}

async fn read_resume_metadata(path: &Path) -> Result<DownloadResumeMetadata> {
    let bytes = fs::read(path).await?;
    Ok(serde_json::from_slice::<DownloadResumeMetadata>(&bytes)?)
}

async fn write_resume_metadata(path: &Path, metadata: &DownloadResumeMetadata) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(metadata)?;
    fs::write(path, bytes).await?;
    Ok(())
}
