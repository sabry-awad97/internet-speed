use crate::core::errors::SpeedTestError;
use crate::core::traits::SpeedTester;
use crate::utils::http_client::HttpClient;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use tokio::time::Instant;

/// UploadTester: single responsibility — measures upload throughput by POSTing bytes.
pub struct UploadTester {
    client: Arc<dyn HttpClient>,
    url: String,
    size_bytes: usize,
}

impl UploadTester {
    pub fn new(client: Arc<dyn HttpClient>, url: impl Into<String>, size_bytes: usize) -> Self {
        Self {
            client,
            url: url.into(),
            size_bytes,
        }
    }
}

#[async_trait]
impl SpeedTester for UploadTester {
    async fn test(&self) -> Result<f64, SpeedTestError> {
        println!(
            "[Upload] Preparing {} MB payload...",
            self.size_bytes / 1_000_000
        );
        // Generate random payload once (keeps responsibility single and test deterministic)
        let payload = {
            // Using pseudo-random bytes via simple PRNG keeps dependency surface small.
            // For cryptographic randomness, use rand::rngs::OsRng.
            let mut v = vec![0u8; self.size_bytes];
            // Simple deterministic fill to avoid extra dependency:
            for (i, byte) in v.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }
            Bytes::from(v)
        };

        println!("[Upload] Uploading to: {}", self.url);
        let start = Instant::now();
        let status = self.client.post_bytes(&self.url, payload).await?;
        let elapsed = start.elapsed().as_secs_f64();
        println!("[Upload] Upload complete in {:.2}s", elapsed);

        if !status.is_success() {
            return Err(SpeedTestError::InvalidResponse(format!(
                "upload endpoint returned status {}",
                status
            )));
        }

        if elapsed <= 0.0 {
            return Err(SpeedTestError::InvalidResponse("zero duration".into()));
        }

        let bits = (self.size_bytes as f64) * 8.0;
        let mbps = bits / (elapsed * 1_000_000.0);
        Ok(mbps)
    }
}
