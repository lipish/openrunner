use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 流式输出事件（内部使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// 增量文本输出
    Token { content: String },
    /// 执行完成
    Done { session_id: Uuid },
    /// 执行出错
    Error { message: String },
}

/// Agent 执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent 类型: claude_code, codex, opencode
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    /// 工作目录
    #[serde(default)]
    pub working_dir: Option<String>,
    /// 超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// 额外参数
    #[serde(default)]
    pub extra_args: Vec<String>,
    /// 模型（可选）
    #[serde(default)]
    pub model: Option<String>,
    /// 环境变量（可选）
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

fn default_agent_type() -> String {
    "claude_code".to_string()
}

fn default_timeout() -> u64 {
    300 // 5 分钟
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: default_agent_type(),
            working_dir: None,
            timeout_secs: default_timeout(),
            extra_args: vec![],
            model: None,
            env: std::collections::HashMap::new(),
        }
    }
}

// ============ API 请求/响应类型 ============

/// 附件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// Run 输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunInput {
    pub text: String,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// Run 元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RunMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_args: Option<Vec<String>>,
}

/// POST /api/runs 请求
#[derive(Debug, Deserialize)]
pub struct CreateRunRequest {
    pub input: RunInput,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub metadata: RunMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub agent_type: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionPayload {
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub messages: Vec<SessionMessage>,
}

#[derive(Debug, Serialize)]
pub struct SessionData {
    pub id: String,
    pub title: String,
    pub agent_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub env: std::collections::HashMap<String, String>,
    pub extra_args: Vec<String>,
    pub hidden: bool,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<SessionMessage>,
}

#[derive(Debug, Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionData>,
}

/// POST /api/runs 响应
#[derive(Debug, Serialize)]
pub struct CreateRunResponse {
    pub run_id: String,
}

/// POST /api/chat 请求
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub agent_type: Option<String>,
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub extra_args: Option<Vec<String>>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// POST /api/chat 响应
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ============ 旧的类型（保持兼容）============

/// API 请求体（旧接口）
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    #[serde(default)]
    pub config: AgentConfig,
}

/// API 响应体（旧接口）
#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub session_id: Uuid,
    pub status: String,
}
