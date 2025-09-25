use hyperware_anthropic_sdk::types::tools::Tool;
use hyperware_anthropic_sdk::{
    AnthropicClient, CacheControl, Content, CreateMessageRequest, Message, Role, SystemPrompt,
    SystemPromptBlock,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let client = AnthropicClient::new(api_key);

    // Example 1: System prompt with cache control
    println!("Example 1: Caching system prompt");
    let request1 = CreateMessageRequest::new(
        "claude-3-5-sonnet-20241022",
        vec![Message {
            role: Role::User,
            content: Content::Text("What is the capital of France?".to_string()),
        }],
        100,
    )
    .with_system_blocks(vec![SystemPromptBlock::text(
        "You are a helpful geography assistant.",
    )
    .with_cache_control(CacheControl::ephemeral())]);

    let response1 = client.send_message(request1).await?;
    println!("Response 1: {:?}", response1.content);
    println!(
        "Cache stats: cache_creation={:?}, cache_read={:?}",
        response1.usage.cache_creation_input_tokens, response1.usage.cache_read_input_tokens
    );

    // Example 2: Tool definitions with cache control
    println!("\nExample 2: Caching tool definitions");
    let weather_tool = Tool::new(
        "get_weather",
        "Get the current weather in a location",
        json!({
            "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA"
            }
        }),
        vec!["location".to_string()],
        None,
    )
    .with_cache_control(CacheControl::ephemeral_1h());

    let request2 = CreateMessageRequest::new(
        "claude-3-5-sonnet-20241022",
        vec![Message {
            role: Role::User,
            content: Content::Text("What's the weather in Paris?".to_string()),
        }],
        100,
    )
    .with_tools(vec![weather_tool]);

    let response2 = client.send_message(request2).await?;
    println!("Response 2: {:?}", response2.content);
    println!(
        "Cache stats: cache_creation={:?}, cache_read={:?}",
        response2.usage.cache_creation_input_tokens, response2.usage.cache_read_input_tokens
    );

    // Example 3: Large context caching
    println!("\nExample 3: Caching large context");
    let large_context =
        "This is a large document that contains important information. ".repeat(100);

    let request3 = CreateMessageRequest::new(
        "claude-3-5-sonnet-20241022",
        vec![Message {
            role: Role::User,
            content: Content::Text(format!(
                "Based on this context: {}\n\nWhat is this document about?",
                large_context
            )),
        }],
        100,
    )
    .with_system_blocks(vec![
        SystemPromptBlock::text("You are an expert document analyzer."),
        SystemPromptBlock::text("Provide concise summaries of documents.")
            .with_cache_control(CacheControl::ephemeral_5m()),
    ]);

    let response3 = client.send_message(request3).await?;
    println!("Response 3: {:?}", response3.content);
    println!(
        "Cache stats: cache_creation={:?}, cache_read={:?}",
        response3.usage.cache_creation_input_tokens, response3.usage.cache_read_input_tokens
    );

    Ok(())
}
