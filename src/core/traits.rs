use crate::core::errors::SpeedTestError;
use async_trait::async_trait;

/// Minimal trait for a speed test.
/// Returns megabits-per-second (Mbps) on success.
#[async_trait]
pub trait SpeedTester: Send + Sync {
    async fn test(&self) -> Result<f64, SpeedTestError>;
}
