use hyperware_anthropic_sdk::AnthropicClient;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // This example demonstrates how to configure custom headers
    
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("Please set ANTHROPIC_API_KEY environment variable");
    
    // Example 1: Add individual custom headers
    let _client = AnthropicClient::new(api_key.clone())
        .with_header("X-Request-ID", "unique-request-123")
        .with_header("X-Custom-Trace", "trace-456");
    
    // Example 2: Add multiple headers at once
    let mut custom_headers = HashMap::new();
    custom_headers.insert("X-Organization-ID".to_string(), "org-789".to_string());
    custom_headers.insert("X-Session-ID".to_string(), "session-abc".to_string());
    custom_headers.insert("User-Agent".to_string(), "MyApp/1.0".to_string());
    
    let client_with_bulk_headers = AnthropicClient::new(api_key)
        .with_headers(custom_headers)
        .with_max_retries(5);
    
    // The custom headers will be included in all API requests
    match client_with_bulk_headers.send_simple_message(
        "claude-3-haiku-20240307",
        "Hello! What's 2+2?",
        100
    ).await {
        Ok(response) => {
            println!("Response: {}", response);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}