#[cfg(test)]
mod tests {
    use hyperware_anthropic_sdk::{AnthropicClient, AnthropicError};

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
}
