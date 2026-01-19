use axum::{
    extract::{Json, Path, Query, State},
    response::{
        sse::{Event, Sse},
        IntoResponse,
    },
    http::StatusCode,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::convert::Infallible;
use serde::Deserialize;

use crate::auth::{self, LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, verify_token, create_token, TOKEN_EXPIRY_SECS};
use crate::types::{
    CreateRunRequest, CreateRunResponse, ChatRequest, ChatResponse,
    ErrorResponse, AgentConfig,
};
use crate::agent::{create_agent, AgentHandle, ClaudeCodeAgent, CodexAgent, OpenCodeAgent, MockAgent, Agent};

use super::AppState;

// ============ Auth Handlers ============

/// POST /api/auth/login
pub async fn login(
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = auth::validate_user(&req.username, &req.password)
        .ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "invalid_credentials".to_string(),
            }))
        })?;

    let token = create_token(&user.id, &user.username, &user.roles)
        .map_err(|_| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "token_generation_failed".to_string(),
            }))
        })?;

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: TOKEN_EXPIRY_SECS,
        refresh_token: None,
        user,
    }))
}

/// POST /api/auth/register
pub async fn register(
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user = auth::register_user(&req.username, &req.password)
        .map_err(|e| {
            (StatusCode::BAD_REQUEST, Json(ErrorResponse {
                error: e,
            }))
        })?;

    Ok(Json(RegisterResponse { user }))
}

// ============ Health Handlers ============

/// GET /health
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true }))
}

/// GET /health/agents
pub async fn health_agents() -> impl IntoResponse {
    let config = AgentConfig::default();

    let claude = ClaudeCodeAgent::new(config.clone());
    let codex = CodexAgent::new(config.clone());
    let opencode = OpenCodeAgent::new(config.clone());
    let mock = MockAgent::new(config.clone());

    let (claude_ok, codex_ok, opencode_ok, mock_ok) = tokio::join!(
        claude.health_check(),
        codex.health_check(),
        opencode.health_check(),
        mock.health_check()
    );

    Json(serde_json::json!({
        "agents": {
            "mock": {
                "available": mock_ok.is_ok(),
                "error": mock_ok.err().map(|e| e.to_string()),
                "description": "Mock agent for testing (no API key required)"
            },
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

/// GET /agents
pub async fn list_agents() -> impl IntoResponse {
    Json(serde_json::json!({
        "agents": [
            {"type": "mock", "description": "Mock agent for testing (no API key required)"},
            {"type": "claude_code", "description": "Claude Code CLI agent"},
            {"type": "codex", "description": "OpenAI Codex CLI agent"},
            {"type": "opencode", "description": "OpenCode CLI agent"}
        ]
    }))
}

// ============ Run Handlers ============

/// POST /api/runs - 创建 Run
pub async fn create_run(
    State(state): State<AppState>,
    Json(req): Json<CreateRunRequest>,
) -> Result<Json<CreateRunResponse>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: 从 Authorization header 获取 user_id
    let user_id = "anonymous";

    let run_id = state.run_manager.create_run(
        user_id,
        req.session_id,
        &req.input.text,
    );

    // 构建 AgentConfig (默认使用 mock agent 便于测试)
    let config = AgentConfig {
        agent_type: req.metadata.agent_type.unwrap_or_else(|| "mock".to_string()),
        working_dir: req.metadata.cwd,
        model: req.metadata.model,
        env: req.metadata.env.unwrap_or_default(),
        ..Default::default()
    };

    // 启动 Run
    if let Err(e) = state.run_manager.start_run(&run_id, config).await {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: e.to_string(),
        })));
    }

    Ok(Json(CreateRunResponse { run_id }))
}

/// SSE query params
#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub access_token: Option<String>,
}

/// 将 RunEvent 转换为 SSE Event
fn run_event_to_sse(event: crate::run::RunEvent) -> Result<Event, Infallible> {
    let event_type = event.event_type();
    let data = event.event_data().to_string();
    Ok(Event::default().event(event_type).data(data))
}

/// GET /api/runs/:run_id/events - 订阅 Run 事件 (SSE)
pub async fn run_events(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Query(query): Query<EventsQuery>,
) -> Result<Sse<std::pin::Pin<Box<dyn futures::stream::Stream<Item = Result<Event, Infallible>> + Send>>>, (StatusCode, Json<ErrorResponse>)> {
    use crate::run::{RunStatus, RunEvent, RunCompleted, RunFailed, CompletedMessage};
    
    // 验证 token（可选）
    if let Some(token) = &query.access_token {
        if verify_token(token).is_err() {
            return Err((StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "invalid_token".to_string(),
            })));
        }
    }

    // 获取 run 信息
    let run = state.run_manager.get_run(&run_id)
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: format!("Run not found: {}", run_id),
            }))
        })?;

    // 如果已完成，返回历史结果
    if run.status == RunStatus::Completed {
        let events = vec![RunEvent::RunCompleted(RunCompleted {
            message: CompletedMessage {
                role: "assistant".to_string(),
                content: run.output.clone(),
                timestamp: run.updated_at,
            },
        })];
        let stream: std::pin::Pin<Box<dyn futures::stream::Stream<Item = Result<Event, Infallible>> + Send>> = 
            Box::pin(futures::stream::iter(events).map(run_event_to_sse));
        return Ok(Sse::new(stream));
    }

    // 如果失败，返回错误
    if run.status == RunStatus::Failed {
        let events = vec![RunEvent::RunFailed(RunFailed {
            error: run.error.unwrap_or_else(|| "Unknown error".to_string()),
        })];
        let stream: std::pin::Pin<Box<dyn futures::stream::Stream<Item = Result<Event, Infallible>> + Send>> = 
            Box::pin(futures::stream::iter(events).map(run_event_to_sse));
        return Ok(Sse::new(stream));
    }

    // 订阅新事件
    let rx = state.run_manager.subscribe(&run_id)
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(ErrorResponse {
                error: format!("Run not found: {}", run_id),
            }))
        })?;

    // 转换为 SSE 流
    let stream: std::pin::Pin<Box<dyn futures::stream::Stream<Item = Result<Event, Infallible>> + Send>> = 
        Box::pin(ReceiverStream::new(rx).map(run_event_to_sse));

    Ok(Sse::new(stream))
}

// ============ Chat Handler (Fallback) ============

/// POST /api/chat - 非流式聊天（降级方案）
pub async fn chat(
    State(_state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, Json<ErrorResponse>)> {
    let config = AgentConfig {
        agent_type: req.agent_type.unwrap_or_else(|| "claude_code".to_string()),
        model: req.model,
        env: req.env.unwrap_or_default(),
        ..Default::default()
    };

    let agent = create_agent(&config)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: e.to_string(),
        })))?;

    let (tx, mut rx) = mpsc::channel::<crate::types::StreamEvent>(100);
    let handle = AgentHandle::spawn(agent, tx);

    // 执行
    let prompt = req.message.clone();
    let run_handle = tokio::spawn(async move {
        handle.run(prompt).await
    });

    // 收集输出
    let mut output = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            crate::types::StreamEvent::Token { content } => output.push_str(&content),
            crate::types::StreamEvent::Done { .. } => break,
            crate::types::StreamEvent::Error { message } => {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: message,
                })));
            }
        }
    }

    let _ = run_handle.await;

    Ok(Json(ChatResponse {
        role: "assistant".to_string(),
        content: output,
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

// 需要引入 StreamExt
use futures::StreamExt;
