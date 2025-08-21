#[cfg(test)]
mod tests {
    use hyperware_anthropic_sdk::{AnthropicClient, AnthropicError};
    use std::collections::HashMap;

    #[test]
    fn test_retry_configuration() {
        // Test that we can configure retry settings
        let _client = AnthropicClient::new("test_key")
            .with_max_retries(5)
            .with_timeout(120);

        // Client should be created successfully with custom settings
        // In a real scenario, this would be tested with a mock server
    }

    #[test]
    fn test_error_types() {
        // Test that ApiError with "overloaded_error" type is properly handled
        let error = AnthropicError::ApiError {
            error_type: "overloaded_error".to_string(),
            message: "The API is currently overloaded".to_string(),
        };

        match error {
            AnthropicError::ApiError { error_type, .. } => {
                assert_eq!(error_type, "overloaded_error");
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn test_custom_headers_single() {
        // Test adding individual custom headers
        let _client = AnthropicClient::new("test_key")
            .with_header("X-Custom-Header", "custom-value")
            .with_header("X-Another-Header", "another-value");

        // Client should be created successfully with custom headers
    }

    #[test]
    fn test_custom_headers_bulk() {
        // Test adding multiple headers at once
        let mut headers = HashMap::new();
        headers.insert("X-Org-ID".to_string(), "org-123".to_string());
        headers.insert("X-Session".to_string(), "session-456".to_string());

        let _client = AnthropicClient::new("test_key").with_headers(headers);

        // Client should be created successfully with bulk headers
    }

    #[test]
    fn test_chained_configuration() {
        // Test that all configuration methods can be chained
        let mut headers = HashMap::new();
        headers.insert("X-Test".to_string(), "test-value".to_string());

        let _client = AnthropicClient::new("test_key")
            .with_base_url("https://custom.api.com")
            .with_api_version("2024-01-01")
            .with_timeout(120)
            .with_max_retries(10)
            .with_header("X-Individual", "individual-value")
            .with_headers(headers);

        // Client should be created with all configurations
    }

    #[test]
    fn test_oauth_configuration() {
        // Test OAuth authentication mode
        let _oauth_client = AnthropicClient::new("bearer_token_here").with_oauth();

        // Client should be created with OAuth mode enabled
    }

    #[test]
    fn test_oauth_with_custom_headers() {
        // Test OAuth mode combined with custom headers
        let mut headers = HashMap::new();
        headers.insert("X-Custom".to_string(), "custom-value".to_string());

        let _oauth_client = AnthropicClient::new("bearer_token")
            .with_oauth()
            .with_headers(headers)
            .with_header("X-Request-ID", "req-123");

        // Client should be created with OAuth and custom headers
    }
}
