# Hyperware Anthropic SDK

A Rust SDK for Hyperware processes to access the Anthropic API, providing seamless integration with Claude models.

## Features

- Full support for Anthropic Messages API
- Tool use capabilities for function calling
- Multi-turn conversation support
- Image input support (base64 and URL)
- Automatic retry and error handling
- Non-streaming mode (optimized for Hyperware processes)
- Type-safe request and response structures

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
hyperware-anthropic-sdk = { git = "https://github.com/hyperware/hyperware-anthropic-sdk" }
```

## Quick Start

This SDK is designed to be used within Hyperware Hyperapps, which provide the async runtime.

```rust
use hyperware_anthropic_sdk::AnthropicClient;

// Within a Hyperapp async context
async fn my_hyperapp_function() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the client with your API key
    let client = AnthropicClient::new("your-api-key");

    // Send a simple message
    let response = client.send_simple_message(
        "claude-opus-4-1-20250805",
        "What is the capital of France?",
        100,
    ).await?;

    println!("Response: {}", response);
    Ok(())
}
```

## API Overview

### Client Initialization

```rust
let client = AnthropicClient::new("api-key")
    .with_base_url("https://custom-api.example.com")  // Optional
    .with_api_version("2023-06-01")                   // Optional
    .with_timeout(120);                                // Optional (seconds)
```

### Simple Text Messages

```rust
// Quick method for simple queries
let response = client.send_simple_message(
    "claude-opus-4-1-20250805",
    "Your prompt here",
    max_tokens,
).await?;
```

### Multi-turn Conversations

```rust
use hyperware_anthropic_sdk::{CreateMessageRequest, Message, Role, Content};

let request = CreateMessageRequest::new(
    "claude-opus-4-1-20250805",
    vec![
        Message {
            role: Role::User,
            content: Content::Text("Hello!".to_string()),
        },
        Message {
            role: Role::Assistant,
            content: Content::Text("Hi there!".to_string()),
        },
        Message {
            role: Role::User,
            content: Content::Text("How are you?".to_string()),
        },
    ],
    1000,
);

let response = client.send_message(request).await?;
```

### Tool Use

```rust
use hyperware_anthropic_sdk::{Tool, ToolChoice};
use serde_json::json;

// Define a tool
let weather_tool = Tool::new(
    "get_weather",
    "Get current weather for a location",
    json!({
        "location": {
            "type": "string",
            "description": "City and state"
        }
    }),
    vec!["location".to_string()],
);

// Create request with tools
let request = CreateMessageRequest::new(model, messages, max_tokens)
    .with_tools(vec![weather_tool])
    .with_tool_choice(ToolChoice::Auto {
        disable_parallel_tool_use: None
    });

// Send and handle tool use in response
let response = client.send_message(request).await?;
```

### System Prompts

```rust
let request = CreateMessageRequest::new(model, messages, max_tokens)
    .with_system("You are a helpful assistant specialized in Rust programming.");
```

### Image Support

```rust
use hyperware_anthropic_sdk::{ContentBlock, ImageSource, ImageSourceData};

let image_block = ContentBlock::Image {
    source: ImageSource {
        source_type: ImageSourceType::Base64,
        data: ImageSourceData::Base64 {
            media_type: "image/jpeg".to_string(),
            data: base64_encoded_image,
        },
    },
    cache_control: None,
};

let message = Message {
    role: Role::User,
    content: Content::Blocks(vec![
        image_block,
        ContentBlock::Text {
            text: "What's in this image?".to_string(),
            cache_control: None,
        },
    ]),
};
```

## Response Handling

```rust
let response = client.send_message(request).await?;

// Access response metadata
println!("Message ID: {}", response.id);
println!("Model: {}", response.model);
println!("Stop reason: {:?}", response.stop_reason);
println!("Tokens used: {} input, {} output",
    response.usage.input_tokens,
    response.usage.output_tokens
);

// Process content blocks
for block in &response.content {
    match block {
        ResponseContentBlock::Text { text, .. } => {
            println!("Text: {}", text);
        }
        ResponseContentBlock::ToolUse { id, name, input } => {
            println!("Tool {} called with ID {}", name, id);
            // Handle tool execution
        }
    }
}
```

## Error Handling

The SDK provides comprehensive error handling:

```rust
use hyperware_anthropic_sdk::AnthropicError;

match client.send_message(request).await {
    Ok(response) => {
        // Handle success
    }
    Err(AnthropicError::ApiError { error_type, message }) => {
        // Handle API errors
        eprintln!("API Error ({}): {}", error_type, message);
    }
    Err(AnthropicError::RateLimit) => {
        // Handle rate limiting
        eprintln!("Rate limit exceeded");
    }
    Err(e) => {
        // Handle other errors
        eprintln!("Error: {}", e);
    }
}
```

## Hyperware Integration

This SDK is designed to work seamlessly with Hyperware processes. The HTTP client functionality is provided by the `hyperware_process_lib::http::client` module, which includes:
- `send_request_await_response` - Async HTTP request function
- `HttpClientError` - Error types for HTTP operations

The SDK requires a Hyperware Hyperapp runtime for async execution.

## Models

The SDK supports all Claude models:
- `claude-opus-4-1-20250805` - Most capable model
- `claude-sonnet-4-20250514` - Balanced performance
- `claude-haiku-3-20240307` - Fast and efficient

## Environment Variables

Set your API key as an environment variable:

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
```

## Conversation Management

The SDK provides a powerful `Conversation` struct for managing ongoing conversations with tool use loops:

### Basic Conversation

```rust
use hyperware_anthropic_sdk::{AnthropicClient, Conversation};

async fn chat_example() -> Result<(), Box<dyn std::error::Error>> {
    let client = AnthropicClient::new("your-api-key");
    let mut conversation = Conversation::new("claude-opus-4-1-20250805", 1000)
        .with_system("You are a helpful assistant");

    // Add a user message and send
    let update = conversation.send_user_message(&client, "Hello! What's 2+2?").await?;
    println!("Claude: {}", update.text());

    // Continue the conversation
    let update = conversation.send_user_message(&client, "Now what's 10 times that?").await?;
    println!("Claude: {}", update.text());

    Ok(())
}
```

### Tool Use Loop

```rust
use hyperware_anthropic_sdk::{AnthropicClient, Conversation, Tool, ToolResult};
use serde_json::json;

async fn tool_loop_example() -> Result<(), Box<dyn std::error::Error>> {
    let client = AnthropicClient::new("your-api-key");

    // Define tools
    let calculator_tool = Tool::new(
        "calculator",
        "Perform mathematical calculations",
        json!({
            "expression": {
                "type": "string",
                "description": "Mathematical expression to evaluate"
            }
        }),
        vec!["expression".to_string()],
    );

    let mut conversation = Conversation::new("claude-opus-4-1-20250805", 1000)
        .with_tools(vec![calculator_tool]);

    // Send a message that will trigger tool use
    conversation.add_user_message("What's 123 * 456 + 789?");

    // Complete the full tool loop
    let updates = conversation.complete_tool_loop(&client, |tool_use| async move {
        match tool_use.name.as_str() {
            "calculator" => {
                // Extract the expression from the tool input
                let expression = tool_use.input["expression"].as_str().unwrap_or("");

                // Simulate calculation (in real code, you'd evaluate safely)
                let result = "56877"; // 123 * 456 + 789

                Ok(ToolResult::success(tool_use.id, result))
            }
            _ => Ok(ToolResult::error(tool_use.id, "Unknown tool"))
        }
    }).await?;

    // Get the final response
    if let Some(final_update) = updates.last() {
        println!("Final answer: {}", final_update.text());
    }

    Ok(())
}
```

### Manual Tool Response Handling

```rust
async fn manual_tool_handling() -> Result<(), Box<dyn std::error::Error>> {
    let client = AnthropicClient::new("your-api-key");
    let mut conversation = Conversation::new("claude-opus-4-1-20250805", 1000)
        .with_tools(vec![weather_tool]);

    // Send initial message
    let update = conversation.send_user_message(&client, "What's the weather in NYC?").await?;

    // Check if tools were requested
    if update.has_tool_uses() {
        for tool_use in update.tool_uses {
            println!("Tool requested: {} with input: {}", tool_use.name, tool_use.input);

            // Execute the tool (your implementation)
            let weather_data = fetch_weather(&tool_use.input).await?;

            // Add the result
            conversation.add_tool_result(
                tool_use.id,
                weather_data,
                false, // not an error
            )?;
        }

        // Send the tool results and get final response
        let final_update = conversation.send(&client).await?;
        println!("Claude's response: {}", final_update.text());
    }

    Ok(())
}
```

### Managing Conversation State

```rust
// Fork a conversation to explore different paths
let alternate_conversation = conversation.fork();

// Access and modify message history
let messages = conversation.messages();
println!("Conversation has {} messages", messages.len());

// Clear and start fresh while keeping settings
conversation.clear();

// Check for pending tool uses
if conversation.has_pending_tool_uses() {
    let pending = conversation.pending_tool_uses();
    println!("{} tools waiting for responses", pending.len());
}
```

## Usage Examples

The SDK is designed for use within Hyperware Hyperapps. All examples assume you're running within a Hyperapp async context.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
