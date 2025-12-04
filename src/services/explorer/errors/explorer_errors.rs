use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExplorerError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Missing input value in transaction")]
    MissingInputValue,
    #[error("Explorer client not initialized")]
    ClientNotInitialized,
    #[error("Failed to build HTTP client: {0}")]
    ClientBuildError(String),
}