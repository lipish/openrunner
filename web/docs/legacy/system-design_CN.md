# Agent UI 系统设计

**基于 Zed Agent 实现**  
**日期：** 2025-10-02  
**版本：** 2.0

## 目录

1. [概述](#概述)
2. [Zed Agent UI 分析](#zed-agent-ui-分析)
3. [系统架构](#系统架构)
4. [组件设计](#组件设计)
5. [交互模式](#交互模式)
6. [技术实现](#技术实现)
7. [数据流](#数据流)
8. [最佳实践](#最佳实践)

---

## 概述

本系统设计基于 Zed Agent 的实际实现，为使用 GPUI 构建 AI 代理界面提供了生产就绪的参考。该设计强调简洁性、性能和以开发者为中心的工作流程。

### 从 Zed Agent 观察到的关键点

**设计理念：**
- 极简、无干扰的界面
- 内联对话流程（无独立面板）
- 代码优先，带语法高亮
- 与编辑器工作流程无缝集成
- 清晰的视觉层次

**核心功能：**
- 流式聊天界面
- 内联代码块与语法高亮
- 可展开详情的工具执行
- 上下文感知（工作区文件）
- 键盘驱动的交互

---

## Zed Agent UI 分析

### 布局结构

根据截图，Zed Agent 使用**单面板、垂直流**设计：

```
┌─────────────────────────────────────────────────────────┐
│  编辑器标签栏                                           │
│  [main.rs] [Assistant] [...]                           │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 👤 用户                                           │ │
│  │ 你能帮我实现一个登录系统吗？                      │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 🤖 助手                                           │ │
│  │                                                   │ │
│  │ 我会帮你创建一个登录系统。让我先搜索代码库。      │ │
│  │                                                   │ │
│  │ ┌─────────────────────────────────────────────┐ │ │
│  │ │ 🔧 search_codebase                          │ │ │
│  │ │ query: "authentication"                     │ │ │
│  │ │ ✓ 找到 3 个文件                             │ │ │
│  │ │ [显示详情 ▼]                                │ │ │
│  │ └─────────────────────────────────────────────┘ │ │
│  │                                                   │ │
│  │ 根据搜索结果，这是实现方法：                      │ │
│  │                                                   │ │
│  │ ```rust                                           │ │
│  │ pub fn login(username: &str, password: &str) {   │ │
│  │     // 实现代码                                  │ │
│  │ }                                                 │ │
│  │ ```                                               │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │ 输入消息...                      [📎] [发送]     │ │
│  └───────────────────────────────────────────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 关键 UI 元素

#### 1. 消息气泡

**用户消息：**
- 左对齐头像图标
- 浅色背景（微妙区分）
- 清晰的排版
- 时间戳（可选，悬停时显示）

**助手消息：**
- 左对齐头像图标
- 略有不同的背景色调
- Markdown 渲染
- 带语法高亮的代码块
- 内联嵌入的工具执行卡片

#### 2. 工具执行卡片

**折叠状态：**
```
┌─────────────────────────────────────┐
│ 🔧 search_codebase          [▼]    │
│ ✓ 在 234ms 内完成                   │
└─────────────────────────────────────┘
```

**展开状态：**
```
┌─────────────────────────────────────┐
│ 🔧 search_codebase          [▲]    │
│                                     │
│ 参数：                              │
│   query: "authentication"           │
│   path: "src/"                      │
│                                     │
│ 结果：                              │
│   • src/auth/login.rs               │
│   • src/auth/session.rs             │
│   • src/middleware/auth.rs          │
│                                     │
│ 耗时：234ms                         │
└─────────────────────────────────────┘
```

#### 3. 代码块

**功能：**
- 语法高亮（Tree-sitter）
- 语言指示器（右上角）
- 复制按钮（悬停时）
- 行号（可选）
- 与消息流内联

```rust
// 渲染示例
┌─────────────────────────────────────┐
│ rust                        [复制]  │
├─────────────────────────────────────┤
│ 1  pub fn login(user: &str) {       │
│ 2      // 实现代码                  │
│ 3  }                                │
└─────────────────────────────────────┘
```

#### 4. 输入区域

**设计：**
- 固定在底部
- 自动扩展的文本区域（最多 5 行）
- 附件按钮（左侧）
- 发送按钮（右侧）
- 键盘快捷键提示（微妙）

```
┌─────────────────────────────────────────────┐
│ 输入消息... (⌘↵ 发送)                      │
│                                             │
│ [📎 附件]                  [发送] 或 [⌘↵]  │
└─────────────────────────────────────────────┘
```

### 视觉设计原则

#### 配色方案

**浅色模式：**
- 背景：`#ffffff`
- 用户消息：`#f5f5f5`
- 助手消息：`#fafafa`
- 工具卡片：`#f0f0f0`
- 代码块：`#f8f8f8`
- 边框：`#e0e0e0`
- 文本：`#1a1a1a`
- 强调色：`#0066cc`

**深色模式：**
- 背景：`#1e1e1e`
- 用户消息：`#2a2a2a`
- 助手消息：`#252525`
- 工具卡片：`#2d2d2d`
- 代码块：`#1a1a1a`
- 边框：`#3a3a3a`
- 文本：`#e0e0e0`
- 强调色：`#4a9eff`

#### 字体排版

- **UI 字体：** 系统默认（macOS 上为 SF Pro）
- **代码字体：** JetBrains Mono / Fira Code
- **基础大小：** 14px
- **行高：** 1.6（提高可读性）
- **代码大小：** 13px

#### 间距

- **消息内边距：** 16px
- **消息间隙：** 12px
- **工具卡片内边距：** 12px
- **代码块内边距：** 12px
- **输入区内边距：** 12px

---

## 系统架构

### 高层架构

```
┌─────────────────────────────────────────────────────────┐
│                    Zed 编辑器                           │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │   编辑器     │  │   助手       │  │   项目       │ │
│  │   面板       │  │   面板       │  │   面板       │ │
│  └──────────────┘  └──────┬───────┘  └──────────────┘ │
│                           │                            │
│         ┌─────────────────┴─────────────────┐          │
│         │                                   │          │
│    ┌────▼────┐                         ┌────▼────┐    │
│    │ 消息    │                         │ 上下文  │    │
│    │ 管理器  │                         │ 管理器  │    │
│    └────┬────┘                         └────┬────┘    │
│         │                                   │          │
│         └─────────────────┬─────────────────┘          │
│                           │                            │
│              ┌────────────▼────────────┐               │
│              │   代理服务              │               │
│              │   - 流式传输            │               │
│              │   - 工具执行            │               │
│              └────────────┬────────────┘               │
└───────────────────────────┼────────────────────────────┘
                            │
                            ▼
              ┌─────────────────────────┐
              │   外部服务              │
              │   - LLM API             │
              │   - LSP 服务器          │
              │   - 文件系统            │
              └─────────────────────────┘
```

### 组件架构

```
AssistantPanel（助手面板）
├── MessageList（消息列表 - VirtualList）
│   ├── UserMessage（用户消息）
│   │   ├── Avatar（头像）
│   │   ├── MessageContent（消息内容 - Markdown）
│   │   └── Timestamp（时间戳）
│   │
│   ├── AssistantMessage（助手消息）
│   │   ├── Avatar（头像）
│   │   ├── MessageContent（消息内容 - Markdown）
│   │   │   ├── TextBlock（文本块）
│   │   │   ├── CodeBlock（代码块 - 带语法高亮）
│   │   │   └── ToolCard（工具卡片）
│   │   │       ├── ToolHeader（工具头部）
│   │   │       ├── ToolParameters（工具参数）
│   │   │       ├── ToolResults（工具结果）
│   │   │       └── ToolStatus（工具状态）
│   │   └── Timestamp（时间戳）
│   │
│   └── StreamingMessage（流式消息）
│       ├── Avatar（头像）
│       ├── StreamingContent（流式内容）
│       └── StreamingIndicator（流式指示器）
│
└── MessageInput（消息输入）
    ├── TextArea（文本区域 - 自动扩展）
    ├── AttachButton（附件按钮）
    └── SendButton（发送按钮）
```

---

## 组件设计

### 1. AssistantPanel（助手面板）

**职责：** 助手界面的主容器

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

### 2. 消息组件

**UserMessage（用户消息）：**
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
                    .child(Text::new("你").weight(600))
                    .child(div().child(self.content))
            )
    }
}
```

**AssistantMessage（助手消息）：**
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

### 3. ToolCard 组件（工具卡片）

**设计：**
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

### 4. CodeBlock 组件（代码块）

**功能：**
- 通过 Tree-sitter 进行语法高亮
- 复制按钮
- 语言指示器
- 行号（可选）

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
                // 头部
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
                // 带语法高亮的代码内容
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

### 5. MessageInput 组件（消息输入）

**功能：**
- 自动扩展的文本区域
- 文件附件
- 键盘快捷键
- 发送按钮

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
                    .placeholder("输入消息... (⌘↵ 发送)")
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
                            .label("附件")
                    )
                    .child(
                        Button::new("send")
                            .primary()
                            .label("发送")
                            .icon(IconName::Send)
                            .disabled(self.text.trim().is_empty())
                    )
            )
    }
}
```

---

## 交互模式

### 1. 消息发送流程

```
用户输入消息
    ↓
用户按 ⌘↵ 或点击发送
    ↓
消息添加到列表（乐观更新）
    ↓
滚动到底部
    ↓
开始流式响应
    ↓
显示流式指示器
    ↓
逐个追加 token
    ↓
内联处理工具调用
    ↓
完成消息
    ↓
清空输入
```

### 2. 工具执行流程

```
代理请求工具执行
    ↓
工具卡片出现（折叠，待处理状态）
    ↓
在后台执行工具
    ↓
更新卡片为"运行中"状态
    ↓
工具完成
    ↓
更新卡片为"成功"状态
    ↓
显示结果预览
    ↓
用户可以展开查看详情
```

### 3. 流式响应模式

**使用 llm-connector：**

```rust
use llm_connector::{Client, ChatRequest, Message};

impl AssistantPanel {
    fn start_streaming(&mut self, cx: &mut Context<Self>) {
        let message_id = MessageId::new();
        self.streaming_message = Some(StreamingMessage::new(message_id));

        // 获取请求的消息
        let llm_messages: Vec<Message> = self.messages
            .iter()
            .map(|m| m.llm_message.clone())
            .collect();

        cx.spawn(|this, mut cx| async move {
            // 初始化 llm-connector 客户端
            let client = Client::from_env();

            // 创建聊天请求
            let request = ChatRequest {
                model: "openai/gpt-4".to_string(),
                messages: llm_messages,
                stream: true,
                ..Default::default()
            };

            // 流式响应
            let mut stream = client.chat_stream(request).await?;

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;

                // 从块中提取内容
                if let Some(choice) = chunk.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        cx.update(|cx| {
                            this.update(cx, |this, cx| {
                                this.append_content(content.clone(), cx);
                            });
                        })?;
                    }

                    // 如果存在，处理工具调用
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

## 技术实现

### 0. LLM API 集成 (llm-connector)

**库：** [llm-connector](https://crates.io/crates/llm-connector)

**为什么选择 llm-connector：**
- ✅ 轻量级且协议无关
- ✅ 支持多个 LLM 提供商（OpenAI、Anthropic 等）
- ✅ 跨不同提供商的统一 API
- ✅ 内置流式传输支持
- ✅ 通过环境变量简单配置
- ✅ 类型安全的 Rust 实现

**设置：**

```toml
# Cargo.toml
[dependencies]
llm-connector = "0.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

**配置：**

```rust
use llm_connector::Client;

// 选项 1：从环境变量
// 在 .env 中设置 LLM_API_KEY 和 LLM_BASE_URL
let client = Client::from_env();

// 选项 2：显式配置
let client = Client::new(
    "https://api.openai.com/v1",
    "your-api-key",
);

// 选项 3：特定提供商
let client = Client::openai("your-api-key");
let client = Client::anthropic("your-api-key");
```

**基本用法：**

```rust
use llm_connector::{Client, ChatRequest, Message};

async fn chat_example() -> Result<()> {
    let client = Client::from_env();

    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message::system("你是一个有帮助的助手。"),
            Message::user("你好！"),
        ],
        stream: false,
        ..Default::default()
    };

    let response = client.chat(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("响应：{}", choice.message.content);
    }

    Ok(())
}
```

**流式用法：**

```rust
use llm_connector::{Client, ChatRequest, Message};
use futures::StreamExt;

async fn streaming_example() -> Result<()> {
    let client = Client::from_env();

    let request = ChatRequest {
        model: "openai/gpt-4".to_string(),
        messages: vec![
            Message::user("给我讲个故事"),
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

**支持的提供商：**

| 提供商 | 模型格式 | 示例 |
|--------|---------|------|
| OpenAI | `openai/model-name` | `openai/gpt-4` |
| Anthropic | `anthropic/model-name` | `anthropic/claude-3-opus` |
| 自定义 | `custom/model-name` | `custom/my-model` |

### 1. 数据模型

**使用 llm-connector 类型：**

```rust
// 从 llm-connector 导入
use llm_connector::{
    Client, ChatRequest, ChatResponse, Message, Choice, Usage,
};

// 应用程序特定的消息包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMessage {
    pub id: MessageId,
    pub llm_message: Message,  // 来自 llm-connector
    pub timestamp: DateTime<Utc>,
    pub tool_calls: Vec<ToolCall>,
}

// 工具调用模型（应用程序特定）
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
    Pending,        // 待处理
    Running,        // 运行中
    Success,        // 成功
    Error(String),  // 错误
}

// 流式消息模型
pub struct StreamingMessage {
    pub id: MessageId,
    pub content: Rope,
    pub tool_calls: Vec<ToolCall>,
}
```

### 2. 代理服务实现

**使用 llm-connector 的完整服务：**

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

    /// 发送聊天请求并获取完整响应
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

    /// 流式聊天响应
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

    /// 更改模型
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }
}

// 全局服务实例
impl Global for AgentService {}
```

### 3. 状态管理

```rust
// 助手上下文
pub struct AssistantContext {
    pub workspace_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub selection: Option<String>,
}

// 全局助手状态
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

### 3. 性能优化

**虚拟滚动：**
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

**增量渲染：**
```rust
impl StreamingMessage {
    pub fn append_chunk(&mut self, text: &str) {
        // 使用 Rope 进行高效的增量更新
        self.content.insert(self.content.len_chars(), text);
    }
}
```

---

## 数据流

### 消息流程图

```
┌─────────────┐
│    用户     │
└──────┬──────┘
       │ 输入消息
       ▼
┌─────────────────┐
│ MessageInput    │
│ （消息输入）    │
└──────┬──────────┘
       │ 发送事件
       ▼
┌─────────────────┐
│ AssistantPanel  │◄──────────┐
│ （助手面板）    │           │
└──────┬──────────┘           │
       │ 添加消息             │
       ▼                      │
┌─────────────────┐           │
│ MessageList     │           │
│ （消息列表）    │           │
└──────┬──────────┘           │
       │ 渲染                 │
       ▼                      │
┌─────────────────┐           │
│ AgentService    │           │
│ （代理服务）    │           │
└──────┬──────────┘           │
       │ 流式响应             │
       ▼                      │
┌─────────────────┐           │
│ StreamHandler   │───────────┘
│ （流处理器）    │
└─────────────────┘
    更新面板
```

### 工具执行流程

```
┌──────────────────┐
│ 代理响应         │
└────────┬─────────┘
         │ 包含工具调用
         ▼
┌──────────────────┐
│ ToolCard         │
│ 状态：待处理     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ ToolExecutor     │
│ （工具执行器）   │
└────────┬─────────┘
         │ 执行
         ▼
┌──────────────────┐
│ ToolCard         │
│ 状态：运行中     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ 工具结果         │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ ToolCard         │
│ 状态：成功       │
│ [显示结果]       │
└──────────────────┘
```

---

## 最佳实践

### 1. UI/UX 最佳实践

**简洁性：**
- 保持界面简洁和专注
- 避免不必要的面板和控件
- 让内容成为主要焦点

**响应性：**
- 立即显示流式响应
- 为所有操作提供视觉反馈
- 适当使用加载状态

**可访问性：**
- 支持键盘导航
- 提供清晰的焦点指示器
- 在适用时使用语义化 HTML/ARIA

### 2. 性能最佳实践

**渲染：**
- 对消息列表使用 VirtualList
- 为流式传输实现增量渲染
- 缓存语法高亮结果
- 对昂贵的操作进行防抖

**内存：**
- 限制内存中的消息历史
- 卸载旧对话
- 一段时间后清除工具结果
- 在适当的地方使用弱引用

### 3. 代码组织

**关注点分离：**
```
ui/
├── assistant_panel.rs    # 主面板
├── message_list.rs       # 消息渲染
├── message_input.rs      # 输入组件
├── tool_card.rs          # 工具执行 UI
└── code_block.rs         # 代码渲染
```

**状态管理：**
```
state/
├── assistant.rs          # 助手状态
├── conversation.rs       # 对话状态
└── context.rs            # 上下文管理
```

**服务：**
```
services/
├── agent.rs              # 代理 API 客户端
├── tools.rs              # 工具执行
└── streaming.rs          # 流处理
```

### 4. 错误处理

```rust
pub enum AssistantError {
    #[error("网络错误：{0}")]
    Network(String),
    
    #[error("API 错误：{0}")]
    API(String),
    
    #[error("工具执行失败：{0}")]
    ToolExecution(String),
}

impl AssistantError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Network(_) => 
                "连接丢失。请检查您的网络连接。".into(),
            Self::API(msg) => 
                format!("API 错误：{}", msg),
            Self::ToolExecution(msg) => 
                format!("工具失败：{}", msg),
        }
    }
}
```

---

## 从 Zed Agent 获得的关键见解

### 1. 内联工具执行

**观察：** 工具执行卡片直接嵌入在消息流中，而不是在单独的面板中。

**优势：**
- 保持对话上下文
- 减少认知负担
- 清晰的因果关系
- 更容易引用工具结果

**实现：**
```rust
pub enum ContentBlock {
    Text(String),
    Code { language: String, code: String },
    Tool(ToolCall),  // 内联嵌入
}
```

### 2. 极简设计

**观察：** 没有复杂的侧边栏，没有多个面板，只有干净的垂直流。

**优势：**
- 减少视觉混乱
- 将注意力集中在对话上
- 更容易实现和维护
- 更适合小屏幕

**设计决策：**
- 单面板布局
- 基于标签的导航（如果需要）
- 上下文操作（不总是可见）

### 3. 代码优先方法

**观察：** 代码块是一等公民，具有出色的语法高亮。

**优势：**
- 完美适合开发者工作流程
- 易于复制和使用代码
- 清晰的视觉区分
- 专业外观

**实现：**
- 使用 Tree-sitter 进行语法高亮
- 语言检测
- 悬停时显示复制按钮
- 正确保留缩进

### 4. 流式 UX

**观察：** 响应逐个 token 出现，滚动流畅。

**优势：**
- 即时反馈
- 感觉响应迅速
- 如果方向错误可以取消
- 自然的对话流程

**技术方法：**
- 使用 Rope 数据结构进行高效更新
- 带平滑动画的自动滚动
- 防抖重新渲染
- 流式传输期间的取消按钮

### 5. 上下文感知

**观察：** 代理可以访问工作区文件和当前编辑器状态。

**优势：**
- 更相关的响应
- 可以引用实际代码
- 理解项目结构
- 更好的工具执行

**实现：**
```rust
pub struct AssistantContext {
    pub workspace_files: Vec<PathBuf>,
    pub active_file: Option<PathBuf>,
    pub selection: Option<String>,
    pub cursor_position: Option<Position>,
}
```

---

## 实现建议

### 1. 从简单开始

**阶段 1：核心聊天（第 1-2 周）**
- 带虚拟滚动的基本消息列表
- 简单的文本输入
- 流式响应显示
- 暂不支持工具执行

**阶段 2：工具支持（第 3-4 周）**
- 添加工具执行框架
- 实现基本工具（read_file、search）
- 工具状态显示
- 可展开的工具卡片

**阶段 3：完善（第 5-6 周）**
- 语法高亮
- 代码块改进
- 更好的错误处理
- 性能优化

### 2. 组件可重用性

**共享组件：**
```rust
// 可重用的消息气泡
pub struct MessageBubble {
    role: Role,
    content: AnyElement,
    timestamp: DateTime<Utc>,
}

// 可重用的代码块
pub struct CodeBlock {
    language: String,
    code: String,
}

// 可重用的工具卡片
pub struct ToolCard {
    tool_call: ToolCall,
}
```

### 3. 状态管理策略

**本地状态：**
- 组件特定的 UI 状态
- 临时输入值
- 展开状态

**全局状态：**
- 对话历史
- 助手设置
- 工作区上下文

**示例：**
```rust
// 组件中的本地状态
pub struct MessageInput {
    text: String,  // 本地
    is_composing: bool,  // 本地
}

// 全局状态
pub struct AssistantState {
    conversations: HashMap<ConversationId, Conversation>,  // 全局
    settings: AssistantSettings,  // 全局
}
```

### 4. 性能优化

**关键优化：**

1. **虚拟滚动**（必须有）
   - 对消息使用 VirtualList
   - 仅渲染可见项
   - 平滑滚动

2. **增量渲染**（必须有）
   - 对流式内容使用 Rope
   - 仅更新更改的部分
   - 防抖重新渲染

3. **语法高亮缓存**（最好有）
   - 缓存高亮的代码
   - 主题更改时失效
   - 后台处理

4. **延迟加载**（最好有）
   - 按需加载旧消息
   - 历史记录分页
   - 卸载屏幕外内容

### 5. 错误处理策略

**面向用户的错误：**
```rust
pub fn handle_error(error: AssistantError, cx: &mut App) {
    let message = error.user_message();
    let actions = error.recovery_actions();

    // 显示通知
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

**开发者错误：**
```rust
// 记录日志用于调试
tracing::error!("工具执行失败：{:?}", error);

// 用于监控的指标
metrics::increment_counter!("assistant.errors", "type" => error.error_type());
```

### 6. 测试策略

**单元测试：**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_message_parsing() {
        let json = r#"{"role":"user","content":"你好"}"#;
        let message: Message = serde_json::from_str(json).unwrap();
        assert_eq!(message.role, Role::User);
    }

    #[test]
    fn test_streaming_append() {
        let mut msg = StreamingMessage::new(MessageId::new());
        msg.append_chunk("你好");
        msg.append_chunk("世界");
        assert_eq!(msg.content.to_string(), "你好世界");
    }
}
```

**集成测试：**
```rust
#[tokio::test]
async fn test_agent_stream() {
    let agent = AgentService::new("test-key");
    let messages = vec![Message::user("你好")];

    let mut stream = agent.chat_stream(messages).await.unwrap();
    let chunks: Vec<_> = stream.collect().await;

    assert!(!chunks.is_empty());
}
```

---

## 对比：Zed Agent vs. 传统聊天 UI

| 方面 | Zed Agent | 传统聊天 UI |
|------|-----------|-------------|
| **布局** | 单面板，垂直流 | 多面板带侧边栏 |
| **工具显示** | 与消息内联 | 单独的面板或模态框 |
| **代码块** | 一等公民，带高亮 | 基本等宽文本 |
| **上下文** | 工作区感知 | 隔离的对话 |
| **导航** | 基于标签（最小化） | 复杂的侧边栏导航 |
| **焦点** | 代码和开发 | 一般对话 |
| **复杂度** | 低（更易实现） | 高（更多功能） |
| **性能** | 优秀（简单布局） | 良好（更多开销） |

**建议：** 对于以开发者为中心的代理 UI，遵循 Zed Agent 方法。它更简单、更专注，更适合编码工作流程。

---

## 结论

本系统设计基于经过验证的 Zed Agent 实现，为使用 GPUI 构建 AI 代理界面提供了坚实的基础。关键原则是：

1. **简洁性**：极简、专注的界面
2. **性能**：虚拟滚动和增量渲染
3. **以开发者为中心**：代码优先，带语法高亮
4. **无缝集成**：在编辑器工作流程中工作
5. **清晰反馈**：透明的工具执行和流式传输

**关键要点：**

✅ **应该做：**
- 保持界面简单和专注
- 使用虚拟滚动提高性能
- 将工具内联嵌入消息中
- 提供出色的代码块渲染
- 立即显示流式响应

❌ **不应该做：**
- 添加复杂的多面板布局
- 创建单独的工具执行面板
- 为了功能牺牲性能
- 隐藏代理正在做什么
- 在操作期间阻塞 UI

遵循这个设计，您可以构建一个生产就绪的 AI 代理界面，在保持高性能的同时提供出色的用户体验。

---

## 任务管理系统

### 概述

任务管理系统允许代理在执行过程中动态追加、修改和取消任务。这为复杂工作流提供了灵活性，使代理能够根据中间结果调整计划。

### 核心概念

#### 1. 任务结构

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
    Pending,      // 尚未开始
    Running,      // 正在执行
    Completed,    // 成功完成
    Failed,       // 执行失败
    Cancelled,    // 手动取消
    Blocked,      // 等待依赖
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

#### 2. 任务管理器

```rust
pub struct TaskManager {
    tasks: HashMap<TaskId, Task>,
    execution_queue: VecDeque<TaskId>,
    active_task: Option<TaskId>,
    listeners: Vec<Box<dyn Fn(TaskEvent) + Send + Sync>>,
}

impl TaskManager {
    /// 追加新任务到队列
    pub fn append_task(&mut self, task: Task) -> Result<TaskId> {
        // 验证任务
        self.validate_task(&task)?;

        // 添加到任务映射
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);

        // 添加到执行队列
        self.execution_queue.push_back(task_id.clone());

        // 通知监听器
        self.notify(TaskEvent::TaskAdded(task_id.clone()));

        Ok(task_id)
    }

    /// 向现有任务追加子任务
    pub fn append_subtask(&mut self, parent_id: TaskId, task: Task) -> Result<TaskId> {
        // 验证父任务存在
        let parent = self.tasks.get_mut(&parent_id)
            .ok_or(Error::TaskNotFound)?;

        // 添加子任务
        let task_id = task.id.clone();
        parent.children.push(task_id.clone());

        // 添加到任务映射
        self.tasks.insert(task_id.clone(), task);

        // 通知监听器
        self.notify(TaskEvent::SubtaskAdded {
            parent_id,
            task_id: task_id.clone(),
        });

        Ok(task_id)
    }

    /// 取消任务及其子任务
    pub fn cancel_task(&mut self, task_id: TaskId) -> Result<()> {
        // 获取任务
        let task = self.tasks.get_mut(&task_id)
            .ok_or(Error::TaskNotFound)?;

        // 检查任务是否可以取消
        if task.status == TaskStatus::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }

        // 取消任务
        task.status = TaskStatus::Cancelled;
        task.updated_at = Utc::now();

        // 递归取消所有子任务
        let children = task.children.clone();
        for child_id in children {
            self.cancel_task(child_id)?;
        }

        // 从执行队列中移除
        self.execution_queue.retain(|id| id != &task_id);

        // 通知监听器
        self.notify(TaskEvent::TaskCancelled(task_id));

        Ok(())
    }

    /// 更新任务状态
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

    /// 获取下一个要执行的任务
    pub fn next_task(&mut self) -> Option<TaskId> {
        while let Some(task_id) = self.execution_queue.pop_front() {
            if let Some(task) = self.tasks.get(&task_id) {
                // 检查任务是否准备好执行
                if task.status == TaskStatus::Pending && self.are_dependencies_met(&task_id) {
                    return Some(task_id);
                }
            }
        }
        None
    }

    /// 检查所有依赖是否满足
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

#### 3. 任务事件

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

### UI 集成

#### 1. 任务列表组件

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
                // 任务头部
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
                    // 取消按钮
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
                                .child("取消")
                        )
                )
            })
    }
}
```

#### 2. 任务追加 UI

在代理执行期间，显示内联 UI 以追加任务：

```
┌─────────────────────────────────────────────────────┐
│ 🤖 助手                                             │
│                                                     │
│ 我正在分析代码库...                                 │
│                                                     │
│ ┌─────────────────────────────────────────────┐   │
│ │ 📋 当前任务                                 │   │
│ │                                             │   │
│ │ ✓ 搜索代码库                                │   │
│ │ ⏳ 分析依赖                                  │   │
│ │ ⏸️  生成实现计划                             │   │
│ │                                             │   │
│ │ [+ 添加任务] [取消全部]                     │   │
│ └─────────────────────────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### 使用场景

#### 1. 动态任务规划

**场景：** 代理在执行过程中发现需要额外工作。

```rust
// 代理正在分析代码
task_manager.append_task(Task {
    name: "分析依赖".to_string(),
    description: "检查 package.json 中的所有依赖".to_string(),
    status: TaskStatus::Pending,
    // ...
});

// 分析过程中发现缺少测试
task_manager.append_task(Task {
    name: "编写缺失的测试".to_string(),
    description: "发现 5 个函数没有测试".to_string(),
    status: TaskStatus::Pending,
    // ...
});
```

#### 2. 子任务分解

**场景：** 将复杂任务分解为更小的步骤。

```rust
let parent_task_id = task_manager.append_task(Task {
    name: "实现登录系统".to_string(),
    // ...
})?;

// 添加子任务
task_manager.append_subtask(parent_task_id, Task {
    name: "创建用户模型".to_string(),
    // ...
})?;

task_manager.append_subtask(parent_task_id, Task {
    name: "实现身份验证".to_string(),
    // ...
})?;

task_manager.append_subtask(parent_task_id, Task {
    name: "添加密码哈希".to_string(),
    // ...
})?;
```

#### 3. 任务取消

**场景：** 用户想要停止长时间运行的操作。

```rust
// 用户点击取消按钮
task_manager.cancel_task(task_id)?;

// 所有子任务自动取消
// 代理优雅地停止执行
```

#### 4. 依赖管理

**场景：** 任务之间相互依赖。

```rust
let task_a = task_manager.append_task(Task {
    name: "从 API 获取数据".to_string(),
    // ...
})?;

let task_b = task_manager.append_task(Task {
    name: "处理数据".to_string(),
    metadata: TaskMetadata {
        dependencies: vec![task_a],
        // ...
    },
    // ...
})?;

// task_b 只会在 task_a 完成后执行
```

### 最佳实践

#### 1. 任务粒度

✅ **应该：**
- 将复杂任务分解为更小、可管理的步骤
- 每个任务应该有清晰、单一的职责
- 提供有意义的任务名称和描述

❌ **不应该：**
- 创建太多微任务（开销大）
- 任务粒度太粗（难以跟踪进度）

#### 2. 错误处理

✅ **应该：**
- 优雅地处理任务失败
- 提供清晰的错误消息
- 允许重试失败的任务

❌ **不应该：**
- 静默失败任务
- 因一个失败阻塞整个工作流

#### 3. 用户反馈

✅ **应该：**
- 显示实时任务进度
- 允许用户取消任务
- 为任务状态提供视觉反馈

❌ **不应该：**
- 隐藏代理正在做什么
- 使任务无法取消
- 显示过多技术细节

### 性能考虑

1. **任务队列管理**
   - 使用高效的数据结构（VecDeque 用于队列）
   - 限制最大队列大小
   - 定期清理已完成的任务

2. **事件通知**
   - 尽可能批量处理事件
   - 防抖 UI 更新
   - 使用异步通知避免阻塞

3. **内存管理**
   - 将旧任务归档到磁盘
   - 限制内存中的任务历史
   - 清理已取消的任务

---

## 下一步

1. **查看 Zed Agent 截图**以理解视觉设计
2. **按照架构设置基本项目结构**
3. **实现核心聊天界面**，支持流式传输
4. **添加工具执行**，内联显示
5. **实现任务管理系统**，支持追加/取消功能
6. **完善 UI**，添加语法高亮和动画
7. **与真实用户测试**并根据反馈迭代

**预估时间线：**
- 第 1-2 周：核心聊天界面
- 第 3-4 周：工具执行
- 第 5-6 周：任务管理系统
- 第 7-8 周：完善和优化
- 第 9-10 周：测试和改进

**总计：** 一个开发者 10 周完成生产就绪的实现。

祝您构建 Agent UI 顺利！🚀

