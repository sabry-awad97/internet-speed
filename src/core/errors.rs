use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpeedTestError {
    #[error("network error: {0}")]
    Network(String),

    #[error("io error: {0}")]
    #[allow(dead_code)]
    Io(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("test aborted: {0}")]
    #[allow(dead_code)]
    Aborted(String),
}
