// Hyperware Anthropic SDK
// A Rust library for Hyperware processes to access the Anthropic API

pub mod client;
pub mod conversation;
pub mod error;
pub mod types;

pub use client::AnthropicClient;
pub use conversation::{Conversation, ConversationUpdate, ToolResult, ToolResultData, PendingToolUse};
pub use error::AnthropicError;
pub use types::*;

// Re-export commonly used types
pub use types::messages::{
    CreateMessageRequest,
    Message,
    MessageResponse,
    Role,
    ContentBlock,
};
pub use types::tools::{Tool, ToolChoice};
