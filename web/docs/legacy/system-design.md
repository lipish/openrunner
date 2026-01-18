# Agent UI System Design

**Based on Zed Agent Implementation**  
**Date:** 2025-10-02  
**Version:** 2.0

## Table of Contents

1. [Overview](#overview)
2. [UI Analysis from Zed Agent](#ui-analysis-from-zed-agent)
3. [System Architecture](#system-architecture)
4. [Component Design](#component-design)
5. [Interaction Patterns](#interaction-patterns)
6. [Technical Implementation](#technical-implementation)
7. [Data Flow](#data-flow)
8. [Best Practices](#best-practices)

---

## Overview

This system design is based on the actual Zed Agent implementation, which provides a production-ready reference for building AI agent interfaces with GPUI. The design emphasizes simplicity, performance, and developer-focused workflows.

### Key Observations from Zed Agent

**Design Philosophy:**
- Minimalist, distraction-free interface
- Inline conversation flow (no separate panels)
- Code-first approach with syntax highlighting
- Seamless integration with editor workflow
- Clear visual hierarchy

**Core Features:**
- Streaming chat interface
- Inline code blocks with syntax highlighting
- Tool execution with expandable details
- Context awareness (workspace files)
- Keyboard-driven interaction

---

## UI Analysis from Zed Agent

### Layout Structure

Based on the screenshots, Zed Agent uses a **single-panel, vertical flow** design:

```
┌─────────────────────────────────────────────────────────┐
│  Editor Tab Bar                                         │
│  [main.rs] [Assistant] [...]                           │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 👤 User                                           │ │
│  │ Can you help me implement a login system?        │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 🤖 Assistant                                      │ │
│  │                                                   │ │
│  │ I'll help you create a login system. Let me      │ │
│  │ search the codebase first.                       │ │
│  │                                                   │ │
│  │ ┌─────────────────────────────────────────────┐ │ │
│  │ │ 🔧 search_codebase                          │ │ │
│  │ │ query: "authentication"                     │ │ │
│  │ │ ✓ Found 3 files                             │ │ │
│  │ │ [Show details ▼]                            │ │ │
│  │ └─────────────────────────────────────────────┘ │ │
│  │                                                   │ │
│  │ Based on the search, here's how to implement:    │ │
│  │                                                   │ │
│  │ ```rust                                           │ │
│  │ pub fn login(username: &str, password: &str) {   │ │
│  │     // Implementation                            │ │
│  │ }                                                 │ │
│  │ ```                                               │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ Type a message...                    [📎] [Send] │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Key UI Elements

#### 1. Message Bubbles

**User Messages:**
- Left-aligned avatar icon
- Light background (subtle differentiation)
- Clean typography
- Timestamp (optional, shown on hover)

**Assistant Messages:**
- Left-aligned avatar icon
- Slightly different background shade
- Markdown rendering
- Code blocks with syntax highlighting
- Tool execution cards embedded inline

#### 2. Tool Execution Cards

**Collapsed State:**
```
┌─────────────────────────────────────┐
│ 🔧 search_codebase          [▼]    │
│ ✓ Completed in 234ms                │
└─────────────────────────────────────┘
```

**Expanded State:**
```
┌─────────────────────────────────────┐
│ 🔧 search_codebase          [▲]    │
│                                     │
│ Parameters:                         │
│   query: "authentication"           │
│   path: "src/"                      │
│                                     │
│ Results:                            │
│   • src/auth/login.rs               │
│   • src/auth/session.rs             │
│   • src/middleware/auth.rs          │
│                                     │
│ Duration: 234ms                     │
└─────────────────────────────────────┘
```

#### 3. Code Blocks

**Features:**
- Syntax highlighting (Tree-sitter)
- Language indicator (top-right)
- Copy button (on hover)
- Line numbers (optional)
- Inline with message flow

```rust
// Example rendering
┌─────────────────────────────────────┐
│ rust                        [Copy]  │
├─────────────────────────────────────┤
│ 1  pub fn login(user: &str) {       │
│ 2      // Implementation            │
│ 3  }                                │
└─────────────────────────────────────┘
```

#### 4. Input Area

**Design:**
- Fixed at bottom
- Auto-expanding textarea (up to 5 lines)
- Attachment button (left)
- Send button (right)
- Keyboard shortcut hint (subtle)

```
┌─────────────────────────────────────────────┐
│ Type a message... (⌘↵ to send)             │
│                                             │
│ [📎 Attach]              [Send] or [⌘↵]    │
└─────────────────────────────────────────────┘
```

### Visual Design Principles

#### Color Scheme

**Light Mode:**
- Background: `#ffffff`
- User message: `#f5f5f5`
- Assistant message: `#fafafa`
- Tool card: `#f0f0f0`
- Code block: `#f8f8f8`
- Border: `#e0e0e0`
- Text: `#1a1a1a`
- Accent: `#0066cc`

**Dark Mode:**
- Background: `#1e1e1e`
- User message: `#2a2a2a`
- Assistant message: `#252525`
- Tool card: `#2d2d2d`
- Code block: `#1a1a1a`
- Border: `#3a3a3a`
- Text: `#e0e0e0`
- Accent: `#4a9eff`

#### Typography

- **UI Font:** System default (SF Pro on macOS)
- **Code Font:** JetBrains Mono / Fira Code
- **Base Size:** 14px
- **Line Height:** 1.6 (for readability)
- **Code Size:** 13px

#### Spacing

- **Message Padding:** 16px
- **Message Gap:** 12px
- **Tool Card Padding:** 12px
- **Code Block Padding:** 12px
- **Input Padding:** 12px

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Zed Editor                           │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   Editor     │  │   Assistant  │  │   Project    │ │
│  │   Panes      │  │   Panel      │  │   Panel      │ │
│  └──────────────┘  └──────┬───────┘  └──────────────┘ │
│                           │                            │
│         ┌─────────────────┴─────────────────┐          │
│         │                                   │          │
│    ┌────▼────┐                         ┌────▼────┐    │
│    │ Message │                         │ Context │    │
│    │ Manager │                         │ Manager │    │
│    └────┬────┘                         └────┬────┘    │
│         │                                   │          │
│         └─────────────────┬─────────────────┘          │
│                           │                            │
│              ┌────────────▼────────────┐               │
│              │   Agent Service         │               │
│              │   - Streaming           │               │
│              │   - Tool Execution      │               │
│              └────────────┬────────────┘               │
└───────────────────────────┼────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │   External Services     │
              │   - LLM API             │
              │   - LSP Servers         │
              │   - File System         │
              └─────────────────────────┘
```

### Component Architecture

```
AssistantPanel
├── MessageList (VirtualList)
│   ├── UserMessage
│   │   ├── Avatar
│   │   ├── MessageContent (Markdown)
│   │   └── Timestamp
│   │
│   ├── AssistantMessage
│   │   ├── Avatar
│   │   ├── MessageContent (Markdown)
│   │   │   ├── TextBlock
│   │   │   ├── CodeBlock (with syntax highlighting)
│   │   │   └── ToolCard
│   │   │       ├── ToolHeader
│   │   │       ├── ToolParameters
│   │   │       ├── ToolResults
│   │   │       └── ToolStatus
│   │   └── Timestamp
│   │
│   └── StreamingMessage
│       ├── Avatar
│       ├── StreamingContent
│       └── StreamingIndicator
│
└── MessageInput
    ├── TextArea (auto-expanding)
    ├── AttachButton
    └── SendButton
```

---

## Component Design

### 1. AssistantPanel

**Responsibility:** Main container for the assistant interface

```rust
pub struct AssistantPanel {
    messages: Vec<Message>,
    streaming_message: Option<StreamingMessage>,
    input_text: String,
    scroll_handle: VirtualListScrollHandle,
    context: AssistantContext,
}

impl AssistantPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            messages: Vec::new(),
            streaming_message: None,
            input_text: String::new(),
            scroll_handle: VirtualListScrollHandle::new(),
            context: AssistantContext::new(),
        }
    }
}
```

### 2. Message Components

**UserMessage:**
```rust
pub struct UserMessage {
    content: String,
    timestamp: DateTime<Utc>,
    attachments: Vec<Attachment>,
}

impl RenderOnce for UserMessage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .gap_3()
            .p_4()
            .child(Avatar::new().icon(IconName::User))
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .child(Text::new("You").weight(600))
                    .child(div().child(self.content))
            )
    }
}
```

**AssistantMessage:**
```rust
pub struct AssistantMessage {
    content: MessageContent,
    timestamp: DateTime<Utc>,
    tool_calls: Vec<ToolCall>,
}

pub enum MessageContent {
    Text(String),
    Markdown(String),
    Mixed(Vec<ContentBlock>),
}

pub enum ContentBlock {
    Text(String),
    Code { language: String, code: String },
    Tool(ToolCall),
}
```

### 3. ToolCard Component

**Design:**
```rust
pub struct ToolCard {
    tool_call: ToolCall,
    expanded: bool,
}

impl ToolCard {
    fn render_collapsed(&self, cx: &App) -> impl IntoElement {
        h_flex()
            .p_3()
            .gap_2()
            .bg(cx.theme().muted.opacity(0.3))
            .rounded_md()
            .items_center()
            .child(Icon::new(IconName::Tool))
            .child(Text::new(&self.tool_call.name).weight(500))
            .child(self.render_status(cx))
            .child(
                Button::new("expand")
                    .icon(IconName::ChevronDown)
                    .ghost()
                    .compact()
            )
    }
    
    fn render_expanded(&self, cx: &App) -> impl IntoElement {
        v_flex()
            .p_3()
            .gap_3()
            .bg(cx.theme().muted.opacity(0.3))
            .rounded_md()
            .child(self.render_header(cx))
            .child(self.render_parameters(cx))
            .child(self.render_results(cx))
    }
}
```

### 4. CodeBlock Component

**Features:**
- Syntax highlighting via Tree-sitter
- Copy button
- Language indicator
- Line numbers (optional)

```rust
pub struct CodeBlock {
    language: String,
    code: String,
    show_line_numbers: bool,
}

impl RenderOnce for CodeBlock {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .rounded_md()
            .overflow_hidden()
            .border_1()
            .border_color(cx.theme().border)
            .child(
                // Header
                h_flex()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .bg(cx.theme().muted.opacity(0.5))
                    .child(Text::new(&self.language).size_sm())
                    .child(
                        Button::new("copy")
                            .icon(IconName::Copy)
                            .ghost()
                            .compact()
                    )
            )
            .child(
                // Code content with syntax highlighting
                div()
                    .p_3()
                    .bg(cx.theme().background)
                    .child(
                        SyntaxHighlighter::new(&self.code, &self.language)
                    )
            )
    }
}
```

### 5. MessageInput Component

**Features:**
- Auto-expanding textarea
- File attachment
- Keyboard shortcuts
- Send button

```rust
pub struct MessageInput {
    text: String,
    attachments: Vec<Attachment>,
}

impl MessageInput {
    fn render(&self, window: &mut Window, cx: &mut Context<AssistantPanel>) -> impl IntoElement {
        v_flex()
            .p_3()
            .gap_2()
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                TextInput::new("message-input")
                    .placeholder("Type a message... (⌘↵ to send)")
                    .value(self.text.clone())
                    .multiline(true)
                    .max_lines(5)
                    .on_change(cx.listener(|this, value, _, _| {
                        this.input_text = value;
                    }))
                    .on_key_down(cx.listener(|this, event, window, cx| {
                        if event.key == "Enter" && event.modifiers.command {
                            this.send_message(window, cx);
                        }
                    }))
            )
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        Button::new("attach")
                            .icon(IconName::Paperclip)
                            .ghost()
                            .label("Attach")
                    )
                    .child(
                        Button::new("send")
                            .primary()
                            .label("Send")
                            .icon(IconName::Send)
                            .disabled(self.text.trim().is_empty())
                    )
            )
    }
}
```

---

## Interaction Patterns

### 1. Message Sending Flow

```
User types message
    ↓
User presses ⌘↵ or clicks Send
    ↓
Message added to list (optimistic update)
    ↓
Scroll to bottom
    ↓
Start streaming response
    ↓
Show streaming indicator
    ↓
Append tokens as they arrive
    ↓
Handle tool calls inline
    ↓
Finalize message
    ↓
Clear input
```

### 2. Tool Execution Flow

```
Agent requests tool execution
    ↓
Tool card appears (collapsed, pending state)
    ↓
Execute tool in background
    ↓
Update card to "running" state
    ↓
Tool completes
    ↓
Update card to "success" state
    ↓
Show results preview
    ↓
User can expand for details
```

### 3. Streaming Response Pattern

**Using llm-connector:**

```rust
use llm_connector::{Client, ChatRequest, Message};

impl AssistantPanel {
    fn start_streaming(&mut self, cx: &mut Context<Self>) {
        let message_id = MessageId::new();
        self.streaming_message = Some(StreamingMessage::new(message_id));

        // Get messages for the request
        let llm_messages: Vec<Message> = self.messages
            .iter()
            .map(|m| m.llm_message.clone())
            .collect();

        cx.spawn(|this, mut cx| async move {
            // Initialize llm-connector client
            let client = Client::from_env();

            // Create chat request
            let request = ChatRequest {
                model: "openai/gpt-4".to_string(),
                messages: llm_messages,
                stream: true,
                ..Default::default()
            };

            // Stream responses
            let mut stream = client.chat_stream(request).await?;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;

                // Extract content from chunk
                if let Some(choice) = chunk.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        cx.update(|cx| {
                            this.update(cx, |this, cx| {
                                this.append_content(content.clone(), cx);
                            });
                        })?;
                    }

                    // Handle tool calls if present
                    if let Some(tool_calls) = &choice.delta.tool_calls {
                        for tool_call in tool_calls {
                            cx.update(|cx| {
                                this.update(cx, |this, cx| {
                                    this.add_tool_call(tool_call.clone(), cx);
                                });
                            })?;
                        }
                    }
                }
            }

            cx.update(|cx| {
                this.update(cx, |this, cx| {
                    this.finalize_streaming(cx);
                });
            })?;

            Ok(())
        }).detach();
    }
}
```

---

## Technical Implementation

### 0. LLM API Integration (llm-connector)

**Library:** [llm-connector](https://crates.io/crates/llm-connector)

**Why llm-connector:**
- ✅ Lightweight and protocol-agnostic
- ✅ Supports multiple LLM providers (OpenAI, Anthropic, etc.)
- ✅ Unified API across different providers
- ✅ Built-in streaming support
- ✅ Simple configuration via environment variables
- ✅ Type-safe Rust implementation

**Setup:**

```toml
# Cargo.toml
[dependencies]
llm-connector = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

**Configuration:**

```rust
use llm_connector::Client;

// Option 1: From environment variables
// Set LLM_API_KEY and LLM_BASE_URL in .env
let client = Client::from_env();

// Option 2: Explicit configuration
let client = Client::new(
    "https://api.openai.com/v1",
    "your-api-key",
);

// Option 3: Provider-specific
let client = Client::openai("your-api-key");
let client = Client::anthropic("your-api-key");
```

**Basic Usage:**

```rust
use llm_connector::{Client, ChatRequest, Message};

async fn chat_example() -> Result<()> {
    let client = Client::from_env();

    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message::system("You are a helpful assistant."),
            Message::user("Hello!"),
        ],
        stream: false,
        ..Default::default()
    };

    let response = client.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }

    Ok(())
}
```

**Streaming Usage:**

```rust
use llm_connector::{Client, ChatRequest, Message};
use futures::StreamExt;

async fn streaming_example() -> Result<()> {
    let client = Client::from_env();

    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message::user("Tell me a story"),
        ],
        stream: true,
        ..Default::default()
    };

    let mut stream = client.chat_stream(request).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
            }
        }
    }

    Ok(())
}
```

**Supported Providers:**

| Provider | Model Format | Example |
|----------|-------------|---------|
| OpenAI | `openai/model-name` | `openai/gpt-4` |
| Anthropic | `anthropic/model-name` | `anthropic/claude-3-opus` |
| Custom | `custom/model-name` | `custom/my-model` |

### 1. Data Models

**Using llm-connector Types:**

```rust
// Import from llm-connector
use llm_connector::{
    Client, ChatRequest, ChatResponse, Message, Choice, Usage,
};

// Application-specific message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMessage {
    pub id: MessageId,
    pub llm_message: Message,  // From llm-connector
    pub timestamp: DateTime<Utc>,
    pub tool_calls: Vec<ToolCall>,
}

// Tool call model (application-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: ToolCallId,
    pub name: String,
    pub parameters: serde_json::Value,
    pub status: ToolStatus,
    pub result: Option<ToolResult>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStatus {
    Pending,
    Running,
    Success,
    Error(String),
}

// Streaming model
pub struct StreamingMessage {
    pub id: MessageId,
    pub content: Rope,
    pub tool_calls: Vec<ToolCall>,
}
```

### 2. Agent Service Implementation

**Complete Service with llm-connector:**

```rust
use llm_connector::{Client, ChatRequest, Message};
use futures::StreamExt;
use anyhow::Result;

pub struct AgentService {
    client: Client,
    model: String,
}

impl AgentService {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            client: Client::from_env(),
            model: model.into(),
        }
    }

    pub fn with_client(client: Client, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    /// Send a chat request and get a complete response
    pub async fn chat(&self, messages: Vec<AppMessage>) -> Result<ChatResponse> {
        let llm_messages: Vec<Message> = messages
            .into_iter()
            .map(|m| m.llm_message)
            .collect();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: llm_messages,
            stream: false,
            ..Default::default()
        };

        self.client.chat(request).await
    }

    /// Stream a chat response
    pub async fn chat_stream(
        &self,
        messages: Vec<AppMessage>,
    ) -> Result<impl Stream<Item = Result<ChatResponse>>> {
        let llm_messages: Vec<Message> = messages
            .into_iter()
            .map(|m| m.llm_message)
            .collect();

        let request = ChatRequest {
            model: self.model.clone(),
            messages: llm_messages,
            stream: true,
            ..Default::default()
        };

        self.client.chat_stream(request).await
    }

    /// Change the model
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }
}

// Global service instance
impl Global for AgentService {}
```

### 3. State Management

```rust
// Assistant context
pub struct AssistantContext {
    pub workspace_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub selection: Option<String>,
}

// Global assistant state
pub struct AssistantState {
    pub conversations: HashMap<ConversationId, Conversation>,
    pub active_conversation: Option<ConversationId>,
    pub settings: AssistantSettings,
    pub agent_service: AgentService,
}

impl Global for AssistantState {}

impl AssistantState {
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            active_conversation: None,
            settings: AssistantSettings::default(),
            agent_service: AgentService::new("openai/gpt-4"),
        }
    }
}
```

### 3. Performance Optimizations

**Virtual Scrolling:**
```rust
v_virtual_list(
    "messages",
    messages.len(),
    move |idx, window, cx| {
        match &messages[idx] {
            Message { role: Role::User, .. } => {
                UserMessage::new(messages[idx].clone()).into_any_element()
            }
            Message { role: Role::Assistant, .. } => {
                AssistantMessage::new(messages[idx].clone()).into_any_element()
            }
            _ => div().into_any_element(),
        }
    },
    window,
    cx,
)
```

**Incremental Rendering:**
```rust
impl StreamingMessage {
    pub fn append_chunk(&mut self, text: &str) {
        // Use Rope for efficient incremental updates
        self.content.insert(self.content.len_chars(), text);
    }
}
```

---

## Data Flow

### Message Flow Diagram

```
┌─────────────┐
│    User     │
└──────┬──────┘
       │ Types message
       ▼
┌─────────────────┐
│ MessageInput    │
└──────┬──────────┘
       │ Send event
       ▼
┌─────────────────┐
│ AssistantPanel  │◄──────────┐
└──────┬──────────┘           │
       │ Add message          │
       ▼                      │
┌─────────────────┐           │
│ MessageList     │           │
└──────┬──────────┘           │
       │ Render               │
       ▼                      │
┌─────────────────┐           │
│ AgentService    │           │
└──────┬──────────┘           │
       │ Stream response      │
       ▼                      │
┌─────────────────┐           │
│ StreamHandler   │───────────┘
└─────────────────┘
    Updates panel
```

### Tool Execution Flow

```
┌──────────────────┐
│ Agent Response   │
└────────┬─────────┘
         │ Contains tool call
         ▼
┌──────────────────┐
│ ToolCard         │
│ Status: Pending  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ ToolExecutor     │
└────────┬─────────┘
         │ Execute
         ▼
┌──────────────────┐
│ ToolCard         │
│ Status: Running  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Tool Result      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ ToolCard         │
│ Status: Success  │
│ [Show Results]   │
└──────────────────┘
```

---

## Best Practices

### 1. UI/UX Best Practices

**Simplicity:**
- Keep the interface minimal and focused
- Avoid unnecessary panels and controls
- Let content be the primary focus

**Responsiveness:**
- Show streaming responses immediately
- Provide visual feedback for all actions
- Use loading states appropriately

**Accessibility:**
- Support keyboard navigation
- Provide clear focus indicators
- Use semantic HTML/ARIA when applicable

### 2. Performance Best Practices

**Rendering:**
- Use VirtualList for message lists
- Implement incremental rendering for streaming
- Cache syntax highlighting results
- Debounce expensive operations

**Memory:**
- Limit message history in memory
- Unload old conversations
- Clear tool results after time
- Use weak references where appropriate

### 3. Code Organization

**Separation of Concerns:**
```
ui/
├── assistant_panel.rs    # Main panel
├── message_list.rs       # Message rendering
├── message_input.rs      # Input component
├── tool_card.rs          # Tool execution UI
└── code_block.rs         # Code rendering
```

**State Management:**
```
state/
├── assistant.rs          # Assistant state
├── conversation.rs       # Conversation state
└── context.rs            # Context management
```

**Services:**
```
services/
├── agent.rs              # Agent API client
├── tools.rs              # Tool execution
└── streaming.rs          # Stream handling
```

### 4. Error Handling

```rust
pub enum AssistantError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("API error: {0}")]
    API(String),
    
    #[error("Tool execution failed: {0}")]
    ToolExecution(String),
}

impl AssistantError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Network(_) => 
                "Connection lost. Please check your internet connection.".into(),
            Self::API(msg) => 
                format!("API Error: {}", msg),
            Self::ToolExecution(msg) => 
                format!("Tool failed: {}", msg),
        }
    }
}
```

---

## Key Insights from Zed Agent

### 1. Inline Tool Execution

**Observation:** Tool execution cards are embedded directly in the message flow, not in a separate panel.

**Benefits:**
- Maintains conversation context
- Reduces cognitive load
- Clear cause-and-effect relationship
- Easier to reference tool results

**Implementation:**
```rust
pub enum ContentBlock {
    Text(String),
    Code { language: String, code: String },
    Tool(ToolCall),  // Embedded inline
}
```

### 2. Minimalist Design

**Observation:** No complex sidebar, no multiple panels, just a clean vertical flow.

**Benefits:**
- Reduces visual clutter
- Focuses attention on conversation
- Easier to implement and maintain
- Better for smaller screens

**Design Decision:**
- Single panel layout
- Tab-based navigation (if needed)
- Contextual actions (not always visible)

### 3. Code-First Approach

**Observation:** Code blocks are first-class citizens with excellent syntax highlighting.

**Benefits:**
- Perfect for developer workflows
- Easy to copy and use code
- Clear visual distinction
- Professional appearance

**Implementation:**
- Tree-sitter for syntax highlighting
- Language detection
- Copy button on hover
- Proper indentation preservation

### 4. Streaming UX

**Observation:** Responses appear token-by-token with smooth scrolling.

**Benefits:**
- Immediate feedback
- Feels responsive
- Can cancel if going wrong direction
- Natural conversation flow

**Technical Approach:**
- Rope data structure for efficient updates
- Auto-scroll with smooth animation
- Debounced re-renders
- Cancel button during streaming

### 5. Context Awareness

**Observation:** Agent has access to workspace files and current editor state.

**Benefits:**
- More relevant responses
- Can reference actual code
- Understands project structure
- Better tool execution

**Implementation:**
```rust
pub struct AssistantContext {
    pub workspace_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub selection: Option<String>,
    pub cursor_position: Option<Position>,
}
```

---

## Implementation Recommendations

### 1. Start Simple

**Phase 1: Core Chat (Week 1-2)**
- Basic message list with virtual scrolling
- Simple text input
- Streaming response display
- No tool execution yet

**Phase 2: Tool Support (Week 3-4)**
- Add tool execution framework
- Implement basic tools (read_file, search)
- Tool status display
- Expandable tool cards

**Phase 3: Polish (Week 5-6)**
- Syntax highlighting
- Code block improvements
- Better error handling
- Performance optimization

### 2. Component Reusability

**Shared Components:**
```rust
// Reusable message bubble
pub struct MessageBubble {
    role: Role,
    content: AnyElement,
    timestamp: DateTime<Utc>,
}

// Reusable code block
pub struct CodeBlock {
    language: String,
    code: String,
}

// Reusable tool card
pub struct ToolCard {
    tool_call: ToolCall,
}
```

### 3. State Management Strategy

**Local State:**
- Component-specific UI state
- Temporary input values
- Expansion states

**Global State:**
- Conversation history
- Assistant settings
- Workspace context

**Example:**
```rust
// Local state in component
pub struct MessageInput {
    text: String,  // Local
    is_composing: bool,  // Local
}

// Global state
pub struct AssistantState {
    conversations: HashMap<ConversationId, Conversation>,  // Global
    settings: AssistantSettings,  // Global
}
```

### 4. Performance Optimization

**Critical Optimizations:**

1. **Virtual Scrolling** (Must have)
   - Use VirtualList for messages
   - Render only visible items
   - Smooth scrolling

2. **Incremental Rendering** (Must have)
   - Use Rope for streaming content
   - Update only changed portions
   - Debounce re-renders

3. **Syntax Highlighting Cache** (Nice to have)
   - Cache highlighted code
   - Invalidate on theme change
   - Background processing

4. **Lazy Loading** (Nice to have)
   - Load old messages on demand
   - Pagination for history
   - Unload off-screen content

### 5. Error Handling Strategy

**User-Facing Errors:**
```rust
pub fn handle_error(error: AssistantError, cx: &mut App) {
    let message = error.user_message();
    let actions = error.recovery_actions();

    // Show notification
    Root::update(window, cx, |root, window, cx| {
        root.show_notification(
            Notification::error(message)
                .actions(actions)
                .duration(Duration::from_secs(5)),
            window,
            cx,
        );
    });
}
```

**Developer Errors:**
```rust
// Log for debugging
tracing::error!("Tool execution failed: {:?}", error);

// Metrics for monitoring
metrics::increment_counter!("assistant.errors", "type" => error.error_type());
```

### 6. Testing Strategy

**Unit Tests:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_message_parsing() {
        let json = r#"{"role":"user","content":"Hello"}"#;
        let message: Message = serde_json::from_str(json).unwrap();
        assert_eq!(message.role, Role::User);
    }

    #[test]
    fn test_streaming_append() {
        let mut msg = StreamingMessage::new(MessageId::new());
        msg.append_chunk("Hello");
        msg.append_chunk(" World");
        assert_eq!(msg.content.to_string(), "Hello World");
    }
}
```

**Integration Tests:**
```rust
use llm_connector::{Client, ChatRequest, Message};

#[tokio::test]
async fn test_llm_connector_stream() {
    let client = Client::from_env();

    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![Message::user("Hello")],
        stream: true,
        ..Default::default()
    };

    let mut stream = client.chat_stream(request).await.unwrap();
    let mut chunks = Vec::new();

    while let Some(chunk) = stream.next().await {
        chunks.push(chunk.unwrap());
    }

    assert!(!chunks.is_empty());
}
```

---

## Comparison: Zed Agent vs. Traditional Chat UI

| Aspect | Zed Agent | Traditional Chat UI |
|--------|-----------|---------------------|
| **Layout** | Single panel, vertical flow | Multi-panel with sidebars |
| **Tool Display** | Inline with messages | Separate panel or modal |
| **Code Blocks** | First-class with highlighting | Basic monospace text |
| **Context** | Workspace-aware | Isolated conversation |
| **Navigation** | Tab-based (minimal) | Complex sidebar navigation |
| **Focus** | Code and development | General conversation |
| **Complexity** | Low (easier to implement) | High (more features) |
| **Performance** | Excellent (simple layout) | Good (more overhead) |

**Recommendation:** For a developer-focused agent UI, follow the Zed Agent approach. It's simpler, more focused, and better suited for coding workflows.

---

## Conclusion

This system design is based on the proven Zed Agent implementation, providing a solid foundation for building AI agent interfaces with GPUI. The key principles are:

1. **Simplicity**: Minimal, focused interface
2. **Performance**: Virtual scrolling and incremental rendering
3. **Developer-focused**: Code-first with syntax highlighting
4. **Seamless integration**: Works within the editor workflow
5. **Clear feedback**: Transparent tool execution and streaming

**Key Takeaways:**

✅ **Do:**
- Keep the interface simple and focused
- Use virtual scrolling for performance
- Embed tools inline with messages
- Provide excellent code block rendering
- Show streaming responses immediately

❌ **Don't:**
- Add complex multi-panel layouts
- Create separate tool execution panels
- Sacrifice performance for features
- Hide what the agent is doing
- Block the UI during operations

By following this design, you can build a production-ready AI agent interface that provides an excellent user experience while maintaining high performance.

---

## Task Management System

### Overview

The task management system allows the agent to dynamically append, modify, and cancel tasks during execution. This provides flexibility for complex workflows where the agent needs to adjust its plan based on intermediate results.

### Core Concepts

#### 1. Task Structure

```rust
#[derive(Clone, Debug)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub parent_id: Option<TaskId>,
    pub children: Vec<TaskId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: TaskMetadata,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Pending,      // Not started yet
    Running,      // Currently executing
    Completed,    // Successfully finished
    Failed,       // Execution failed
    Cancelled,    // Manually cancelled
    Blocked,      // Waiting for dependencies
}

#[derive(Clone, Debug)]
pub struct TaskMetadata {
    pub tool_calls: Vec<ToolCall>,
    pub dependencies: Vec<TaskId>,
    pub estimated_duration: Option<Duration>,
    pub actual_duration: Option<Duration>,
    pub error: Option<String>,
}
```

#### 2. Task Manager

```rust
pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
    execution_queue: VecDeque<TaskId>,
    active_task: Option<TaskId>,
    listeners: Vec<Box<dyn Fn(TaskEvent) + Send + Sync>>,
}

impl TaskManager {
    /// Append a new task to the queue
    pub fn append_task(&mut self, task: Task) -> Result<TaskId> {
        // Validate task
        self.validate_task(&task)?;

        // Add to task map
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);

        // Add to execution queue
        self.execution_queue.push_back(task_id.clone());

        // Notify listeners
        self.notify(TaskEvent::TaskAdded(task_id.clone()));

        Ok(task_id)
    }

    /// Append a subtask to an existing task
    pub fn append_subtask(&mut self, parent_id: TaskId, task: Task) -> Result<TaskId> {
        // Validate parent exists
        let parent = self.tasks.get_mut(&parent_id)
            .ok_or(Error::TaskNotFound)?;

        // Add subtask
        let task_id = task.id.clone();
        parent.children.push(task_id.clone());

        // Add to task map
        self.tasks.insert(task_id.clone(), task);

        // Notify listeners
        self.notify(TaskEvent::SubtaskAdded {
            parent_id,
            task_id: task_id.clone(),
        });

        Ok(task_id)
    }

    /// Cancel a task and its subtasks
    pub fn cancel_task(&mut self, task_id: TaskId) -> Result<()> {
        // Get task
        let task = self.tasks.get_mut(&task_id)
            .ok_or(Error::TaskNotFound)?;

        // Check if task can be cancelled
        if task.status == TaskStatus::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }

        // Cancel task
        task.status = TaskStatus::Cancelled;
        task.updated_at = Utc::now();

        // Cancel all subtasks recursively
        let children = task.children.clone();
        for child_id in children {
            self.cancel_task(child_id)?;
        }

        // Remove from execution queue
        self.execution_queue.retain(|id| id != &task_id);

        // Notify listeners
        self.notify(TaskEvent::TaskCancelled(task_id));

        Ok(())
    }

    /// Update task status
    pub fn update_task_status(&mut self, task_id: TaskId, status: TaskStatus) -> Result<()> {
        let task = self.tasks.get_mut(&task_id)
            .ok_or(Error::TaskNotFound)?;

        task.status = status.clone();
        task.updated_at = Utc::now();

        self.notify(TaskEvent::TaskStatusChanged {
            task_id,
            status,
        });

        Ok(())
    }

    /// Get next task to execute
    pub fn next_task(&mut self) -> Option<TaskId> {
        while let Some(task_id) = self.execution_queue.pop_front() {
            if let Some(task) = self.tasks.get(&task_id) {
                // Check if task is ready to execute
                if task.status == TaskStatus::Pending && self.are_dependencies_met(&task_id) {
                    return Some(task_id);
                }
            }
        }
        None
    }

    /// Check if all dependencies are met
    fn are_dependencies_met(&self, task_id: &TaskId) -> bool {
        if let Some(task) = self.tasks.get(task_id) {
            for dep_id in &task.metadata.dependencies {
                if let Some(dep_task) = self.tasks.get(dep_id) {
                    if dep_task.status != TaskStatus::Completed {
                        return false;
                    }
                }
            }
        }
        true
    }
}
```

#### 3. Task Events

```rust
#[derive(Clone, Debug)]
pub enum TaskEvent {
    TaskAdded(TaskId),
    SubtaskAdded {
        parent_id: TaskId,
        task_id: TaskId,
    },
    TaskStarted(TaskId),
    TaskStatusChanged {
        task_id: TaskId,
        status: TaskStatus,
    },
    TaskCompleted {
        task_id: TaskId,
        result: TaskResult,
    },
    TaskFailed {
        task_id: TaskId,
        error: String,
    },
    TaskCancelled(TaskId),
    TaskProgress {
        task_id: TaskId,
        progress: f32,
        message: String,
    },
}
```

### UI Integration

#### 1. Task List Component

```rust
pub struct TaskListView {
    task_manager: Model<TaskManager>,
    expanded_tasks: HashSet<TaskId>,
}

impl TaskListView {
    fn render_task(&self, task: &Task, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                // Task header
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(self.render_status_icon(&task.status))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight(500.))
                            .child(task.name.clone())
                    )
                    .child(self.render_task_actions(task, cx))
            )
            .when(!task.children.is_empty(), |this| {
                this.child(
                    div()
                        .pl_4()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(
                            task.children.iter().filter_map(|child_id| {
                                self.task_manager.read(cx).get_task(child_id)
                                    .map(|child| self.render_task(child, cx))
                            })
                        )
                )
            })
    }

    fn render_task_actions(&self, task: &Task, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .gap_1()
            .when(task.status == TaskStatus::Pending || task.status == TaskStatus::Running, |this| {
                this.child(
                    // Cancel button
                    div()
                        .id(("cancel-task", task.id.clone()))
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(rgb(0xdc3545))
                        .cursor_pointer()
                        .hover(|this| this.bg(rgb(0xc82333)))
                        .on_click({
                            let task_id = task.id.clone();
                            cx.listener(move |this, _, _, cx| {
                                this.task_manager.update(cx, |manager, _| {
                                    manager.cancel_task(task_id.clone()).ok();
                                });
                            })
                        })
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0xffffff))
                                .child("Cancel")
                        )
                )
            })
    }
}
```

#### 2. Task Append UI

During agent execution, show an inline UI to append tasks:

```
┌─────────────────────────────────────────────────────┐
│ 🤖 Assistant                                        │
│                                                     │
│ I'm analyzing the codebase...                      │
│                                                     │
│ ┌─────────────────────────────────────────────┐   │
│ │ 📋 Current Tasks                            │   │
│ │                                             │   │
│ │ ✓ Search codebase                           │   │
│ │ ⏳ Analyze dependencies                      │   │
│ │ ⏸️  Generate implementation plan             │   │
│ │                                             │   │
│ │ [+ Add Task] [Cancel All]                   │   │
│ └─────────────────────────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Use Cases

#### 1. Dynamic Task Planning

**Scenario:** Agent discovers additional work needed during execution.

```rust
// Agent is analyzing code
task_manager.append_task(Task {
    name: "Analyze dependencies".to_string(),
    description: "Check all dependencies in package.json".to_string(),
    status: TaskStatus::Pending,
    // ...
});

// During analysis, discovers missing tests
task_manager.append_task(Task {
    name: "Write missing tests".to_string(),
    description: "Found 5 functions without tests".to_string(),
    status: TaskStatus::Pending,
    // ...
});
```

#### 2. Subtask Decomposition

**Scenario:** Break down a complex task into smaller steps.

```rust
let parent_task_id = task_manager.append_task(Task {
    name: "Implement login system".to_string(),
    // ...
})?;

// Add subtasks
task_manager.append_subtask(parent_task_id, Task {
    name: "Create user model".to_string(),
    // ...
})?;

task_manager.append_subtask(parent_task_id, Task {
    name: "Implement authentication".to_string(),
    // ...
})?;

task_manager.append_subtask(parent_task_id, Task {
    name: "Add password hashing".to_string(),
    // ...
})?;
```

#### 3. Task Cancellation

**Scenario:** User wants to stop a long-running operation.

```rust
// User clicks cancel button
task_manager.cancel_task(task_id)?;

// All subtasks are automatically cancelled
// Agent stops execution gracefully
```

#### 4. Dependency Management

**Scenario:** Tasks depend on each other.

```rust
let task_a = task_manager.append_task(Task {
    name: "Fetch data from API".to_string(),
    // ...
})?;

let task_b = task_manager.append_task(Task {
    name: "Process data".to_string(),
    metadata: TaskMetadata {
        dependencies: vec![task_a],
        // ...
    },
    // ...
})?;

// task_b will only execute after task_a completes
```

### Best Practices

#### 1. Task Granularity

✅ **Do:**
- Break down complex tasks into smaller, manageable steps
- Each task should have a clear, single responsibility
- Provide meaningful task names and descriptions

❌ **Don't:**
- Create too many micro-tasks (overhead)
- Make tasks too coarse-grained (hard to track progress)

#### 2. Error Handling

✅ **Do:**
- Handle task failures gracefully
- Provide clear error messages
- Allow retry for failed tasks

❌ **Don't:**
- Silently fail tasks
- Block the entire workflow on one failure

#### 3. User Feedback

✅ **Do:**
- Show real-time task progress
- Allow users to cancel tasks
- Provide visual feedback for task status

❌ **Don't:**
- Hide what the agent is doing
- Make tasks uncancellable
- Show too much technical detail

### Performance Considerations

1. **Task Queue Management**
   - Use efficient data structures (VecDeque for queue)
   - Limit maximum queue size
   - Clean up completed tasks periodically

2. **Event Notifications**
   - Batch events when possible
   - Debounce UI updates
   - Use async notifications to avoid blocking

3. **Memory Management**
   - Archive old tasks to disk
   - Limit in-memory task history
   - Clean up cancelled tasks

---

## Next Steps

1. **Review the Zed Agent screenshots** to understand the visual design
2. **Set up the basic project structure** following the architecture
3. **Implement the core chat interface** with streaming support
4. **Add tool execution** with inline display
5. **Implement task management system** with append/cancel capabilities
6. **Polish the UI** with syntax highlighting and animations
7. **Test with real users** and iterate based on feedback

**Estimated Timeline:**
- Week 1-2: Core chat interface
- Week 3-4: Tool execution
- Week 5-6: Task management system
- Week 7-8: Polish and optimization
- Week 9-10: Testing and refinement

**Total:** 10 weeks for a production-ready implementation with one developer.

Good luck building your Agent UI! 🚀

