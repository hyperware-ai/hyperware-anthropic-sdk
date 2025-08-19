use crate::error::{AnthropicError, ApiErrorResponse};
use crate::types::messages::{CreateMessageRequest, MessageResponse, Message, Role, Content};
use hyperware_process_lib::http::client::send_request_await_response;
use hyperware_process_lib::http::Method;
use hyperware_process_lib::hyperapp::sleep;
use serde_json;
use std::collections::HashMap;

const ANTHROPIC_API_BASE_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_API_VERSION: &str = "2023-06-01";
const DEFAULT_TIMEOUT_SECONDS: u64 = 60;
const MAX_RETRIES: u32 = 10;
const INITIAL_RETRY_DELAY_MS: u64 = 1000;
const MAX_RETRY_DELAY_MS: u64 = 60000;

pub struct AnthropicClient {
    api_key: String,
    base_url: String,
    api_version: String,
    timeout: u64,
    max_retries: u32,
}

impl AnthropicClient {
    /// Create a new Anthropic API client with the provided API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: ANTHROPIC_API_BASE_URL.to_string(),
            api_version: ANTHROPIC_API_VERSION.to_string(),
            timeout: DEFAULT_TIMEOUT_SECONDS,
            max_retries: MAX_RETRIES,
        }
    }

    /// Create a new client with custom base URL (useful for testing or proxies)
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set a custom API version
    pub fn with_api_version(mut self, api_version: impl Into<String>) -> Self {
        self.api_version = api_version.into();
        self
    }

    /// Set custom timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum number of retries for transient errors
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Calculate retry delay with exponential backoff and jitter, in ms
    fn calculate_retry_delay(attempt: u32) -> u64 {
        let base_delay = INITIAL_RETRY_DELAY_MS * 2u64.pow(attempt);
        let delay_with_jitter = base_delay + (rand::random::<u64>() % 1000);
        let final_delay = delay_with_jitter.min(MAX_RETRY_DELAY_MS);
        final_delay
    }

    /// Check if an error is retryable
    fn is_retryable_error(error: &AnthropicError) -> bool {
        match error {
            AnthropicError::ApiError { error_type, .. } => {
                // Retry on overloaded errors
                error_type == "overloaded_error" || error_type == "api_error"
            }
            AnthropicError::RateLimit => true,
            AnthropicError::HttpClient(msg) => {
                // Retry on connection errors or timeouts
                msg.contains("timeout") || msg.contains("connection")
            }
            _ => false,
        }
    }

    /// Send a message to the Anthropic API with retry logic
    pub async fn send_message(&self, request: CreateMessageRequest) -> Result<MessageResponse, AnthropicError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.send_message_internal(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    // Check if the error is retryable
                    if Self::is_retryable_error(&error) && attempt < self.max_retries {
                        let delay = Self::calculate_retry_delay(attempt);
                        eprintln!("Retrying after error: {}. Attempt {} of {}. Waiting {:?}",
                                 error, attempt + 1, self.max_retries, delay);
                        sleep(delay).await.unwrap();
                        last_error = Some(error);
                    } else {
                        // Non-retryable error or max retries reached
                        return Err(error);
                    }
                }
            }
        }

        // Should not reach here, but return last error if we do
        Err(last_error.unwrap_or_else(||
            AnthropicError::InvalidResponse("Max retries reached".to_string())
        ))
    }

    /// Internal method to send a message without retry logic
    async fn send_message_internal(&self, request: CreateMessageRequest) -> Result<MessageResponse, AnthropicError> {
        // Ensure streaming is disabled
        let mut request = request;
        request.stream = Some(false);

        // Serialize the request body
        let body = serde_json::to_vec(&request)
            .map_err(|e| AnthropicError::Serialization(e.to_string()))?;

        // Build the URL
        let url = format!("{}/v1/messages", self.base_url);
        let url = url::Url::parse(&url)
            .map_err(|_| AnthropicError::InvalidResponse(format!("Invalid URL: {}", url)))?;

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.api_key.clone());
        headers.insert("anthropic-version".to_string(), self.api_version.clone());
        headers.insert("content-type".to_string(), "application/json".to_string());

        // Make the HTTP request using the Hyperware HTTP client
        let response = send_request_await_response(
            Method::POST,
            url,
            Some(headers),
            self.timeout,
            body,
        )
        .await
        .map_err(|e| AnthropicError::HttpClient(e.to_string()))?;

        // Check response status
        let status = response.status();
        let body = response.into_body();

        if status.is_success() {
            // Parse successful response
            serde_json::from_slice::<MessageResponse>(&body)
                .map_err(|e| AnthropicError::Deserialization(format!("Failed to parse response: {}", e)))
        } else {
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_slice::<ApiErrorResponse>(&body) {
                Err(AnthropicError::ApiError {
                    error_type: error_response.error.error_type,
                    message: error_response.error.message,
                })
            } else {
                // Fallback to generic error
                let error_text = String::from_utf8_lossy(&body);
                Err(AnthropicError::InvalidResponse(format!(
                    "API returned status {}: {}",
                    status, error_text
                )))
            }
        }
    }

    /// Create a simple text message request
    pub fn create_simple_message(
        &self,
        model: impl Into<String>,
        prompt: impl Into<String>,
        max_tokens: u32,
    ) -> CreateMessageRequest {
        CreateMessageRequest::new(
            model,
            vec![Message {
                role: Role::User,
                content: Content::Text(prompt.into()),
            }],
            max_tokens,
        )
    }

    /// Send a simple text message and get the response text
    pub async fn send_simple_message(
        &self,
        model: impl Into<String>,
        prompt: impl Into<String>,
        max_tokens: u32,
    ) -> Result<String, AnthropicError> {
        let request = self.create_simple_message(model, prompt, max_tokens);
        let response = self.send_message(request).await?;

        // Extract text from the first content block
        if let Some(first_block) = response.content.first() {
            match first_block {
                crate::types::messages::ResponseContentBlock::Text { text, .. } => Ok(text.clone()),
                _ => Err(AnthropicError::InvalidResponse("Expected text response".to_string())),
            }
        } else {
            Err(AnthropicError::InvalidResponse("Empty response content".to_string()))
        }
    }
}
