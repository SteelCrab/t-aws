use std::path::PathBuf;

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::error::Result;
use crate::platform::Platform;

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
}

impl DownloadProgress {
    pub fn percentage(&self) -> Option<f64> {
        self.total
            .map(|t| (self.downloaded as f64 / t as f64) * 100.0)
    }
}

pub async fn download_installer<F>(platform: &Platform, progress_callback: F) -> Result<PathBuf>
where
    F: Fn(DownloadProgress) + Send + Sync,
{
    let url = platform.download_url();
    let filename = platform.installer_filename();
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(filename);

    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.error_for_status()?;

    let total_size = response.content_length();
    let mut stream = response.bytes_stream();

    let mut file = tokio::fs::File::create(&file_path).await?;
    let mut downloaded: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        progress_callback(DownloadProgress {
            downloaded,
            total: total_size,
        });
    }

    file.flush().await?;
    Ok(file_path)
}
