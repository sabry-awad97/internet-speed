use crate::core::errors::SpeedTestError;
use async_trait::async_trait;
use bytes::Bytes;
use reqwest::{Response, StatusCode};

/// Abstract HttpClient so services don't depend on reqwest directly.
/// This is the DIP in action.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// GET a streaming response (for download test)
    async fn get_stream(&self, url: &str) -> Result<Response, SpeedTestError>;

    /// POST bytes to a URL (for upload test)
    async fn post_bytes(&self, url: &str, body: Bytes) -> Result<StatusCode, SpeedTestError>;
}

/// Concrete implementation using reqwest
pub struct ReqwestHttpClient {
    client: reqwest::Client,
}

impl ReqwestHttpClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("internet-speed-rs/0.1")
            .connect_timeout(std::time::Duration::from_secs(10))
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .expect("failed to build http client");
        Self { client }
    }
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn get_stream(&self, url: &str) -> Result<Response, SpeedTestError> {
        println!("[HTTP] Connecting to: {}", url);
        let resp = self.client.get(url).send().await.map_err(|e| {
            eprintln!("[HTTP] Connection error: {}", e);
            SpeedTestError::Network(format!("Failed to connect: {}", e))
        })?;

        if resp.status().is_success() {
            println!("[HTTP] Connected successfully, status: {}", resp.status());
            Ok(resp)
        } else {
            Err(SpeedTestError::InvalidResponse(format!(
                "HTTP status: {}",
                resp.status()
            )))
        }
    }

    async fn post_bytes(
        &self,
        url: &str,
        body: Bytes,
    ) -> Result<reqwest::StatusCode, SpeedTestError> {
        self.client
            .post(url)
            .body(body)
            .send()
            .await
            .map_err(|e| SpeedTestError::Network(e.to_string()))
            .map(|resp| resp.status())
    }
}
