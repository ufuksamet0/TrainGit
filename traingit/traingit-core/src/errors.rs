//! Error types for TrainGit

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Chrono parse error: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("API error: {0}")]
    ApiError(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;