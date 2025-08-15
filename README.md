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

## Usage Examples

The SDK is designed for use within Hyperware Hyperapps. All examples assume you're running within a Hyperapp async context.

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.