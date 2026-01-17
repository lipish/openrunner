use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 流式输出事件
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
}

fn default_timeout() -> u64 {
    300 // 5 分钟
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: "claude_code".to_string(),
            working_dir: None,
            timeout_secs: default_timeout(),
            extra_args: vec![],
        }
    }
}

/// API 请求体
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub prompt: String,
    #[serde(default)]
    pub config: AgentConfig,
}

/// API 响应体
#[derive(Debug, Serialize)]
pub struct RunResponse {
    pub session_id: Uuid,
    pub status: String,
}
