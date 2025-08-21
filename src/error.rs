use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export HttpClientError from hyperware_process_lib for convenience
pub use hyperware_process_lib::http::client::HttpClientError;

#[derive(Clone, Debug, Error, Serialize, Deserialize)]
pub enum AnthropicError {
    #[error("HTTP client error: {0}")]
    HttpClient(String),

    #[error("API key not provided")]
    MissingApiKey,

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("API error: {error_type}: {message}")]
    ApiError { error_type: String, message: String },

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Authentication failed")]
    Authentication,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

impl From<serde_json::Error> for AnthropicError {
    fn from(err: serde_json::Error) -> Self {
        AnthropicError::Serialization(err.to_string())
    }
}

// API Error response structure from Anthropic
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetail,
    #[serde(rename = "type")]
    pub error_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
