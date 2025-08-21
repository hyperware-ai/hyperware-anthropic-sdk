use crate::client::AnthropicClient;
use crate::error::AnthropicError;
use crate::types::messages::{
    Content, ContentBlock, CreateMessageRequest, Message, MessageResponse, ResponseContentBlock,
    Role, ToolResultContent,
};
use serde_json::Value;

/// Manages an ongoing conversation with Claude, handling message history and tool use loops
#[derive(Debug, Clone)]
pub struct Conversation {
    /// The message history
    messages: Vec<Message>,
    /// The model to use for this conversation
    model: String,
    /// Default max tokens for responses
    max_tokens: u32,
    /// System prompt if any
    system: Option<String>,
    /// Available tools for this conversation
    tools: Option<Vec<crate::types::tools::Tool>>,
    /// Tool choice configuration
    tool_choice: Option<crate::types::tools::ToolChoice>,
    /// Temperature setting
    temperature: Option<f32>,
    /// Track pending tool uses that need responses
    pending_tool_uses: Vec<PendingToolUse>,
}

#[derive(Debug, Clone)]
pub struct PendingToolUse {
    pub id: String,
    pub name: String,
    pub input: Value,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(model: impl Into<String>, max_tokens: u32) -> Self {
        Self {
            messages: Vec::new(),
            model: model.into(),
            max_tokens,
            system: None,
            tools: None,
            tool_choice: None,
            temperature: None,
            pending_tool_uses: Vec::new(),
        }
    }

    /// Set the system prompt
    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Set available tools
    pub fn with_tools(mut self, tools: Vec<crate::types::tools::Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Set tool choice
    pub fn with_tool_choice(mut self, tool_choice: crate::types::tools::ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(Message {
            role: Role::User,
            content: Content::Text(content.into()),
        });
        self
    }

    /// Add a user message with content blocks (for images, etc.)
    pub fn add_user_blocks(&mut self, blocks: Vec<ContentBlock>) -> &mut Self {
        self.messages.push(Message {
            role: Role::User,
            content: Content::Blocks(blocks),
        });
        self
    }

    /// Add an assistant message (useful for providing examples or continuing conversations)
    pub fn add_assistant_message(&mut self, content: impl Into<String>) -> &mut Self {
        self.messages.push(Message {
            role: Role::Assistant,
            content: Content::Text(content.into()),
        });
        self
    }

    /// Add an assistant message with content blocks
    pub fn add_assistant_blocks(&mut self, blocks: Vec<ContentBlock>) -> &mut Self {
        self.messages.push(Message {
            role: Role::Assistant,
            content: Content::Blocks(blocks),
        });
        self
    }

    /// Process a response from Claude and update the conversation state
    pub fn process_response(&mut self, response: &MessageResponse) -> ConversationUpdate {
        let mut tool_uses = Vec::new();
        let mut text_responses = Vec::new();
        let mut blocks = Vec::new();

        // Process each content block in the response
        for block in &response.content {
            match block {
                ResponseContentBlock::Text { text, .. } => {
                    text_responses.push(text.clone());
                    blocks.push(ContentBlock::Text {
                        text: text.clone(),
                        cache_control: None,
                    });
                }
                ResponseContentBlock::ToolUse { id, name, input } => {
                    let pending = PendingToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    };
                    tool_uses.push(pending.clone());
                    self.pending_tool_uses.push(pending);

                    blocks.push(ContentBlock::ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                        cache_control: None,
                    });
                }
            }
        }

        // Add the assistant's response to the conversation
        if !blocks.is_empty() {
            self.add_assistant_blocks(blocks);
        }

        ConversationUpdate {
            tool_uses,
            text_responses,
            stop_reason: response.stop_reason.clone(),
        }
    }

    /// Add tool results for pending tool uses
    pub fn add_tool_results(&mut self, results: Vec<ToolResult>) -> Result<(), AnthropicError> {
        if results.is_empty() {
            return Ok(());
        }

        let mut blocks = Vec::new();

        for result in results {
            // Find and remove the pending tool use
            let pending_index = self
                .pending_tool_uses
                .iter()
                .position(|p| p.id == result.tool_use_id)
                .ok_or_else(|| {
                    AnthropicError::InvalidResponse(format!(
                        "No pending tool use with id: {}",
                        result.tool_use_id
                    ))
                })?;

            self.pending_tool_uses.remove(pending_index);

            // Create the tool result block
            blocks.push(ContentBlock::ToolResult {
                tool_use_id: result.tool_use_id,
                content: Some(match result.content {
                    ToolResultData::Text(text) => ToolResultContent::Text(text),
                    ToolResultData::Blocks(blocks) => ToolResultContent::Blocks(blocks),
                }),
                is_error: Some(result.is_error),
                cache_control: None,
            });
        }

        // Add all tool results as a single user message
        self.add_user_blocks(blocks);
        Ok(())
    }

    /// Convenience method to add a single tool result
    pub fn add_tool_result(
        &mut self,
        tool_use_id: String,
        content: impl Into<String>,
        is_error: bool,
    ) -> Result<(), AnthropicError> {
        self.add_tool_results(vec![ToolResult {
            tool_use_id,
            content: ToolResultData::Text(content.into()),
            is_error,
        }])
    }

    /// Check if there are pending tool uses that need responses
    pub fn has_pending_tool_uses(&self) -> bool {
        !self.pending_tool_uses.is_empty()
    }

    /// Get the list of pending tool uses
    pub fn pending_tool_uses(&self) -> &[PendingToolUse] {
        &self.pending_tool_uses
    }

    /// Build a request from the current conversation state
    pub fn build_request(&self) -> CreateMessageRequest {
        let mut request =
            CreateMessageRequest::new(self.model.clone(), self.messages.clone(), self.max_tokens);

        if let Some(ref system) = self.system {
            request = request.with_system(system.clone());
        }

        if let Some(ref tools) = self.tools {
            request = request.with_tools(tools.clone());
        }

        if let Some(ref tool_choice) = self.tool_choice {
            request = request.with_tool_choice(tool_choice.clone());
        }

        if let Some(temperature) = self.temperature {
            request = request.with_temperature(temperature);
        }

        request
    }

    /// Send the current conversation to Claude and get a response
    pub async fn send(
        &mut self,
        client: &AnthropicClient,
    ) -> Result<ConversationUpdate, AnthropicError> {
        let request = self.build_request();
        let response = client.send_message(request).await?;
        Ok(self.process_response(&response))
    }

    /// Add a user message and immediately send to Claude
    pub async fn send_user_message(
        &mut self,
        client: &AnthropicClient,
        message: impl Into<String>,
    ) -> Result<ConversationUpdate, AnthropicError> {
        self.add_user_message(message);
        self.send(client).await
    }

    /// Complete a full tool use loop: send message, execute tools, send results, get final response
    pub async fn complete_tool_loop<F, Fut>(
        &mut self,
        client: &AnthropicClient,
        mut tool_executor: F,
    ) -> Result<Vec<ConversationUpdate>, AnthropicError>
    where
        F: FnMut(PendingToolUse) -> Fut,
        Fut: std::future::Future<Output = Result<ToolResult, AnthropicError>>,
    {
        let mut updates = Vec::new();

        loop {
            let update = self.send(client).await?;
            let has_tools = !update.tool_uses.is_empty();
            updates.push(update);

            if !has_tools {
                break; // No more tool uses, we're done
            }

            // Execute all pending tools
            let mut results = Vec::new();
            for tool_use in self.pending_tool_uses.clone() {
                let result = tool_executor(tool_use).await?;
                results.push(result);
            }

            // Add the results back to the conversation
            self.add_tool_results(results)?;
        }

        Ok(updates)
    }

    /// Get the current message history
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get a mutable reference to the message history (for advanced use cases)
    pub fn messages_mut(&mut self) -> &mut Vec<Message> {
        &mut self.messages
    }

    /// Clear the conversation history
    pub fn clear(&mut self) {
        self.messages.clear();
        self.pending_tool_uses.clear();
    }

    /// Create a new conversation with the same settings but empty history
    pub fn fork(&self) -> Self {
        Self {
            messages: Vec::new(),
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            system: self.system.clone(),
            tools: self.tools.clone(),
            tool_choice: self.tool_choice.clone(),
            temperature: self.temperature,
            pending_tool_uses: Vec::new(),
        }
    }
}

/// Result of processing a Claude response
#[derive(Debug, Clone)]
pub struct ConversationUpdate {
    /// Tool uses requested by Claude
    pub tool_uses: Vec<PendingToolUse>,
    /// Text responses from Claude
    pub text_responses: Vec<String>,
    /// The stop reason for this response
    pub stop_reason: Option<crate::types::messages::StopReason>,
}

impl ConversationUpdate {
    /// Check if this update contains tool use requests
    pub fn has_tool_uses(&self) -> bool {
        !self.tool_uses.is_empty()
    }

    /// Get the combined text response
    pub fn text(&self) -> String {
        self.text_responses.join("\n")
    }
}

/// A tool execution result to be sent back to Claude
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: ToolResultData,
    pub is_error: bool,
}

#[derive(Debug, Clone)]
pub enum ToolResultData {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl ToolResult {
    /// Create a successful tool result with text content
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultData::Text(content.into()),
            is_error: false,
        }
    }

    /// Create an error tool result
    pub fn error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultData::Text(error.into()),
            is_error: true,
        }
    }
}
