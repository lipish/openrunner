use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// SSE 事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEvent {
    /// 增量消息
    MessageDelta(MessageDelta),
    /// 工具调用开始
    ToolCallStarted(ToolCallStarted),
    /// 工具调用完成
    ToolCallFinished(ToolCallFinished),
    /// Run 完成
    RunCompleted(RunCompleted),
    /// Run 失败
    RunFailed(RunFailed),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallStarted {
    pub tool_call_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFinished {
    pub tool_call_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCompleted {
    pub message: CompletedMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunFailed {
    pub error: String,
}

impl RunEvent {
    /// 获取事件类型名称（用于 SSE event: 字段）
    pub fn event_type(&self) -> &'static str {
        match self {
            RunEvent::MessageDelta(_) => "message_delta",
            RunEvent::ToolCallStarted(_) => "tool_call_started",
            RunEvent::ToolCallFinished(_) => "tool_call_finished",
            RunEvent::RunCompleted(_) => "run_completed",
            RunEvent::RunFailed(_) => "run_failed",
        }
    }

    /// 获取事件数据（用于 SSE data: 字段）
    pub fn event_data(&self) -> serde_json::Value {
        match self {
            RunEvent::MessageDelta(d) => serde_json::json!({ "delta": d.delta }),
            RunEvent::ToolCallStarted(t) => serde_json::json!({
                "tool_call_id": t.tool_call_id,
                "name": t.name,
                "input": t.input
            }),
            RunEvent::ToolCallFinished(t) => serde_json::json!({
                "tool_call_id": t.tool_call_id,
                "output": t.output,
                "ok": t.ok
            }),
            RunEvent::RunCompleted(c) => serde_json::json!({
                "message": {
                    "role": c.message.role,
                    "content": c.message.content,
                    "timestamp": c.message.timestamp.to_rfc3339()
                }
            }),
            RunEvent::RunFailed(f) => serde_json::json!({ "error": f.error }),
        }
    }
}
