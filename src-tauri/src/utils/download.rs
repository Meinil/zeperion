use std::{fs::File, io::Write};

use std::error::Error;

use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Window;

use crate::mapper;
use crate::vo::ServerResourceVo;
use crate::config::{RB, CONFIG};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Download {
    pub id: u32,
    pub content_length: String,
    pub progress_percent: i32
}

pub async fn download(window: Window, item: ServerResourceVo) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let response = client.get(item.download_url.unwrap())
        .send()
        .await?;
    let content_length = response.content_length().unwrap();
    let length_mb = format!("{:.2}MB", content_length / 1024 / 1024);
    let mut buffer = Vec::with_capacity(content_length as usize);
    let mut download_size: u64 = 0;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        buffer.extend(&chunk);
        download_size += chunk.len() as u64;
        let progress_percent = (download_size as f32 / content_length as f32 * 100.0).ceil() as i32;
        window.emit(&format!("download-{}", item.id), Download {
            id: item.id, 
            content_length: length_mb.clone(), 
            progress_percent: progress_percent
        })?;
    }
    let path = format!("{}/{}-{}.jar", CONFIG.get_mirrors_path(), item.brand, item.version);
    let mut file = File::create(path)?;
    file.write_all(&buffer)?;

    // 设置该项已经下载
    let mut rb = RB.acquire().await.unwrap();
    mapper::update_server_item_download(&mut rb, item.id, "1").await?;
    Ok(())
}