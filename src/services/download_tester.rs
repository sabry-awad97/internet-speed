use crate::core::errors::SpeedTestError;
use crate::core::traits::SpeedTester;
use crate::utils::http_client::HttpClient;
use std::sync::Arc;
use tokio::time::Instant;

/// DownloadTester: single responsibility — measures download throughput.
pub struct DownloadTester {
    client: Arc<dyn HttpClient>,
    url: String,
    // optional bytes cap for the test (None = stream until server closes)
    max_bytes: Option<usize>,
}

impl DownloadTester {
    pub fn new(client: Arc<dyn HttpClient>, url: impl Into<String>) -> Self {
        Self {
            client,
            url: url.into(),
            max_bytes: None,
        }
    }

    pub fn with_max_bytes(mut self, bytes: usize) -> Self {
        self.max_bytes = Some(bytes);
        self
    }
}

#[async_trait::async_trait]
impl SpeedTester for DownloadTester {
    async fn test(&self) -> Result<f64, SpeedTestError> {
        println!("[Download] Starting test from: {}", self.url);
        let resp = self.client.get_stream(&self.url).await?;
        println!("[Download] Connected, streaming data...");
        let mut stream = resp.bytes_stream();

        let start = Instant::now();
        let mut downloaded: usize = 0usize;
        let mut last_print = 0usize;

        use futures_util::StreamExt;
        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| {
                eprintln!("[Download] Stream error after {} bytes: {}", downloaded, e);
                SpeedTestError::Network(format!("Stream error: {}", e))
            })?;
            downloaded += chunk.len();

            // Print progress every 1MB
            if downloaded - last_print >= 1_000_000 {
                let elapsed = start.elapsed().as_secs_f64();
                let mbps = (downloaded as f64 * 8.0) / (elapsed * 1_000_000.0);
                println!(
                    "[Download] {} MB downloaded, current speed: {:.2} Mbps",
                    downloaded / 1_000_000,
                    mbps
                );
                last_print = downloaded;
            }

            if let Some(max) = self.max_bytes
                && downloaded >= max
            {
                println!("[Download] Reached max bytes limit: {} bytes", downloaded);
                break;
            }
        }

        let elapsed = start.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return Err(SpeedTestError::InvalidResponse("zero duration".into()));
        }

        // bits per second -> megabits per second
        let mbps = (downloaded as f64 * 8.0) / (elapsed * 1_000_000.0);
        Ok(mbps)
    }
}
