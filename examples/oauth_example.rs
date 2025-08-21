use hyperware_anthropic_sdk::AnthropicClient;

#[tokio::main]
async fn main() {
    // This example demonstrates OAuth authentication mode

    // Get your OAuth Bearer token (this would typically come from an OAuth flow)
    let bearer_token = std::env::var("ANTHROPIC_BEARER_TOKEN")
        .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
        .expect("Please set ANTHROPIC_BEARER_TOKEN or ANTHROPIC_API_KEY environment variable");

    // Create a client using OAuth mode
    let oauth_client = AnthropicClient::new(bearer_token.clone())
        .with_oauth() // Enable OAuth mode - uses Bearer token instead of API key
        .with_max_retries(3);

    println!("Using OAuth authentication with Bearer token...");

    // Send a message using OAuth authentication
    match oauth_client
        .send_simple_message(
            "claude-3-haiku-20240307",
            "Hello! Can you confirm you received this message via OAuth?",
            100,
        )
        .await
    {
        Ok(response) => {
            println!("Response: {}", response);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    // You can also combine OAuth with custom headers
    let oauth_with_headers = AnthropicClient::new(bearer_token)
        .with_oauth()
        .with_header("X-Request-ID", "oauth-request-123")
        .with_header("X-Client-Version", "1.0.0");

    println!("\nUsing OAuth with custom headers...");

    match oauth_with_headers
        .send_simple_message("claude-3-haiku-20240307", "What's 2+2?", 50)
        .await
    {
        Ok(response) => {
            println!("Response: {}", response);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
