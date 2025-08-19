use hyperware_anthropic_sdk::{AnthropicClient, AnthropicError};

#[tokio::main]
async fn main() {
    // This example demonstrates the retry mechanism for handling "Overloaded" API errors

    // Create a client with your API key
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("Please set ANTHROPIC_API_KEY environment variable");

    // Create client with custom retry settings
    let client = AnthropicClient::new(api_key)
        .with_max_retries(3)  // Will retry up to 3 times for retryable errors
        .with_timeout(30);     // 30 second timeout per request

    // Send a message - if the API returns "Overloaded" error, it will automatically retry
    match client.send_simple_message(
        "claude-3-haiku-20240307",
        "What is 2 + 2?",
        100
    ).await {
        Ok(response) => {
            println!("Response: {}", response);
        }
        Err(AnthropicError::ApiError { error_type, message }) => {
            // If we still get an error after retries, handle it here
            if error_type == "overloaded_error" {
                println!("API is still overloaded after retries: {}", message);
            } else {
                println!("API error: {}: {}", error_type, message);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
