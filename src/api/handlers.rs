use axum::{
    extract::Json,
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    http::StatusCode,
};
use futures::stream::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::convert::Infallible;

use crate::agent::{create_agent, AgentHandle};
use crate::types::{RunRequest, StreamEvent};

/// 健康检查
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "openrunner"
    }))
}

/// 运行 Agent（流式 SSE 响应）
pub async fn run_agent(
    Json(req): Json<RunRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    // 创建 agent
    let agent = create_agent(&req.config)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    // 创建流式输出 channel
    let (tx, rx) = mpsc::channel::<StreamEvent>(100);
    
    // 启动 agent
    let handle = AgentHandle::spawn(agent, tx);
    let session_id = handle.session_id;
    
    tracing::info!(session_id = %session_id, prompt = %req.prompt, "Starting agent");
    
    // 异步执行
    let prompt = req.prompt.clone();
    tokio::spawn(async move {
        if let Err(e) = handle.run(prompt).await {
            tracing::error!(session_id = %session_id, error = %e, "Agent execution failed");
        }
    });
    
    // 转换为 SSE 流
    let stream = ReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok(Event::default().data(data))
    });
    
    Ok(Sse::new(stream))
}

/// 同步运行 Agent（等待完成后返回完整结果）
pub async fn run_agent_sync(
    Json(req): Json<RunRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let agent = create_agent(&req.config)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(100);
    let handle = AgentHandle::spawn(agent, tx);
    let session_id = handle.session_id;
    
    // 执行
    let prompt = req.prompt.clone();
    let run_handle = tokio::spawn(async move {
        handle.run(prompt).await
    });
    
    // 收集所有输出
    let mut output = String::new();
    let mut error: Option<String> = None;
    
    while let Some(event) = rx.recv().await {
        match event {
            StreamEvent::Token { content } => output.push_str(&content),
            StreamEvent::Done { .. } => break,
            StreamEvent::Error { message } => {
                error = Some(message);
                break;
            }
        }
    }
    
    // 等待执行完成
    let _ = run_handle.await;
    
    if let Some(err) = error {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, err));
    }
    
    Ok(Json(serde_json::json!({
        "session_id": session_id,
        "output": output,
        "status": "completed"
    })))
}

/// 列出可用的 Agent 类型
pub async fn list_agents() -> impl IntoResponse {
    Json(serde_json::json!({
        "agents": [
            {"type": "claude_code", "description": "Claude Code CLI agent"},
            {"type": "codex", "description": "OpenAI Codex CLI agent"},
            {"type": "opencode", "description": "OpenCode CLI agent"}
        ]
    }))
}

/// 检查各 Agent 的可用状态
pub async fn health_agents() -> impl IntoResponse {
    use crate::agent::{ClaudeCodeAgent, CodexAgent, OpenCodeAgent};
    use crate::types::AgentConfig;
    
    let config = AgentConfig::default();
    
    let claude = ClaudeCodeAgent::new(config.clone());
    let codex = CodexAgent::new(config.clone());
    let opencode = OpenCodeAgent::new(config.clone());
    
    // 并发检查所有 agent
    let (claude_ok, codex_ok, opencode_ok) = tokio::join!(
        claude.health_check(),
        codex.health_check(),
        opencode.health_check()
    );
    
    Json(serde_json::json!({
        "agents": {
            "claude_code": {
                "available": claude_ok.is_ok(),
                "error": claude_ok.err().map(|e| e.to_string()),
                "install": "npm install -g @anthropic-ai/claude-code"
            },
            "codex": {
                "available": codex_ok.is_ok(),
                "error": codex_ok.err().map(|e| e.to_string()),
                "install": "npm install -g @openai/codex"
            },
            "opencode": {
                "available": opencode_ok.is_ok(),
                "error": opencode_ok.err().map(|e| e.to_string()),
                "install": "go install github.com/opencode-ai/opencode@latest"
            }
        }
    }))
}

// 需要引入 StreamExt
use futures::StreamExt;
use crate::agent::Agent;
