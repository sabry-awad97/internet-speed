use crate::core::errors::SpeedTestError;
use crate::core::traits::SpeedTester;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct SpeedResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
}

pub struct SpeedService {
    download: Arc<dyn SpeedTester>,
    upload: Arc<dyn SpeedTester>,
}

impl SpeedService {
    pub fn new(download: Arc<dyn SpeedTester>, upload: Arc<dyn SpeedTester>) -> Self {
        Self { download, upload }
    }

    /// Orchestrate both tests in parallel and return consolidated result.
    pub async fn run(&self) -> Result<SpeedResult, SpeedTestError> {
        // Run both testers concurrently (Tokio join) to keep orchestration simple.
        let (d_res, u_res) = tokio::join!(self.download.test(), self.upload.test());

        let download_mbps = d_res?;
        let upload_mbps = u_res?;

        Ok(SpeedResult {
            download_mbps,
            upload_mbps,
        })
    }
}
