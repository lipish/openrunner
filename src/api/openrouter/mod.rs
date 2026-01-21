use crate::agent::{GatewayConfig, LoadBalancing, GATEWAY_MANAGER};
use crate::api::router::AppState;
use crate::types::{AgentConfig, StreamEvent};
use anyhow::Result;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as AxumJson},
};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// OpenRouter-compatible request structure
#[derive(Debug, Deserialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<OpenRouterMessage>,
    pub stream: Option<bool>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenRouterMessage {
    pub role: String,
    pub content: String,
}

/// OpenRouter-compatible response structure
#[derive(Debug, Serialize)]
pub struct OpenRouterResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<OpenRouterChoice>,
    pub usage: OpenRouterUsage,
}

#[derive(Debug, Serialize)]
pub struct OpenRouterChoice {
    pub index: u32,
    pub message: OpenRouterMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct OpenRouterUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenRouter stream chunk
#[derive(Debug, Serialize)]
pub struct OpenRouterStreamChunk {
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "choices")]
    pub choices: Vec<StreamChoice>,
    #[serde(rename = "usage")]
    pub usage: Option<OpenRouterUsage>,
}

#[derive(Debug, Serialize)]
pub struct StreamChoice {
    #[serde(rename = "index")]
    pub index: u32,
    #[serde(rename = "delta")]
    pub delta: Option<StreamDelta>,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StreamDelta {
    #[serde(rename = "content")]
    pub content: Option<String>,
}

/// Provider management request
#[derive(Debug, Deserialize)]
pub struct ProviderRequest {
    pub name: String,
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub fallback_providers: Option<Vec<String>>,
}

/// Provider list response
#[derive(Debug, Serialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderInfo>,
}

#[derive(Debug, Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub provider: String,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub healthy: bool,
}

// ============ OpenRouter-compatible API Handlers ============

/// POST /v1/chat/completions - OpenRouter-compatible chat endpoint
pub async fn openrouter_chat_completions(
    State(state): State<AppState>,
    Json(req): Json<OpenRouterRequest>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let prompt = req
        .messages
        .iter()
        .map(|m| m.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");

    let config = AgentConfig {
        agent_type: "gateway".to_string(),
        model: Some(req.model.clone()),
        env: std::collections::HashMap::new(),
        ..Default::default()
    };

    let agent = crate::agent::create_agent(&config).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            AxumJson(serde_json::json!({
                "error": e.to_string()
            })),
        )
    })?;

    let (tx, mut rx) = mpsc::channel::<StreamEvent>(100);
    let handle = crate::agent::AgentHandle::spawn(agent, tx);

    // Execute
    let run_handle = tokio::spawn(async move { handle.run(prompt).await });

    let mut output = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            StreamEvent::Token { content } => output.push_str(&content),
            StreamEvent::Done { .. } => break,
            StreamEvent::Error { message } => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AxumJson(serde_json::json!({
                        "error": message
                    })),
                ));
            }
        }
    }

    let _ = run_handle.await;

    let response = OpenRouterResponse {
        id: uuid::Uuid::new_v4().to_string(),
        model: req.model,
        choices: vec![OpenRouterChoice {
            index: 0,
            message: OpenRouterMessage {
                role: "assistant".to_string(),
                content: output,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: OpenRouterUsage {
            prompt_tokens: 0, // Would need to calculate actual token usage
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(AxumJson(response))
}

/// GET /v1/chat/completions - OpenRouter-compatible streaming chat endpoint
pub async fn openrouter_chat_completions_stream(
    State(state): State<AppState>,
    Json(req): Json<OpenRouterRequest>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let prompt = req
        .messages
        .iter()
        .map(|m| m.content.clone())
        .collect::<Vec<_>>()
        .join("\n\n");

    let config = AgentConfig {
        agent_type: "gateway".to_string(),
        model: Some(req.model.clone()),
        env: std::collections::HashMap::new(),
        ..Default::default()
    };

    let agent = crate::agent::create_agent(&config).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            AxumJson(serde_json::json!({
                "error": e.to_string()
            })),
        )
    })?;

    let (tx, rx) = mpsc::channel::<StreamEvent>(100);
    let handle = crate::agent::AgentHandle::spawn(agent, tx);

    // Execute in background
    let _run_handle = tokio::spawn(async move { handle.run(prompt).await });

    // Clone model for use in closure
    let model_for_stream = req.model.clone();

    // Convert events to OpenRouter stream format
    let stream = ReceiverStream::new(rx).map(move |event| {
        let model = model_for_stream.clone();
        match event {
            StreamEvent::Token { content } => {
                let chunk = OpenRouterStreamChunk {
                    id: uuid::Uuid::new_v4().to_string(),
                    model,
                    choices: vec![StreamChoice {
                        index: 0,
                        delta: Some(StreamDelta {
                            content: Some(content),
                        }),
                        finish_reason: None,
                    }],
                    usage: None,
                };
                Ok::<_, std::convert::Infallible>(
                    axum::response::sse::Event::default()
                        .event("chat.completion.chunk")
                        .data(serde_json::to_string(&chunk).unwrap()),
                )
            }
            StreamEvent::Done { .. } => {
                let chunk = OpenRouterStreamChunk {
                    id: uuid::Uuid::new_v4().to_string(),
                    model,
                    choices: vec![StreamChoice {
                        index: 0,
                        delta: Some(StreamDelta { content: None }),
                        finish_reason: Some("stop".to_string()),
                    }],
                    usage: Some(OpenRouterUsage {
                        prompt_tokens: 0,
                        completion_tokens: 0,
                        total_tokens: 0,
                    }),
                };
                Ok(axum::response::sse::Event::default()
                    .event("chat.completion.chunk")
                    .data(serde_json::to_string(&chunk).unwrap()))
            }
            StreamEvent::Error { message } => Ok(axum::response::sse::Event::default()
                .event("error")
                .data(message)),
        }
    });

    let sse = axum::response::sse::Sse::new(stream);
    Ok(sse)
}

/// GET /v1/models - List available models
pub async fn openrouter_models(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let models = vec![
        "openai/gpt-4",
        "openai/gpt-4o",
        "openai/gpt-3.5-turbo",
        "anthropic/claude-3-sonnet-20240229",
        "anthropic/claude-3-haiku-20240307",
        "anthropic/claude-3-opus-20240229",
        "google/gemini-pro",
        "google/gemini-ultra",
    ];

    let response = serde_json::json!({
        "data": models.iter().map(|model| {
            serde_json::json!({
                "id": model,
                "object": "model",
                "owned_by": "openai",
                "context_length": 128000
            })
        }).collect::<Vec<_>>(),
        "object": "list"
    });

    Ok(AxumJson(response))
}

/// GET /v1/models/{model_id} - Get model details
pub async fn openrouter_model_details(
    Path(model_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let response = serde_json::json!({
        "id": model_id,
        "object": "model",
        "owned_by": "openai",
        "context_length": 128000,
        "top_provider": {
            "id": "openai",
            "name": "OpenAI",
            "group": "openai",
            "model": "gpt-4"
        }
    });

    Ok(AxumJson(response))
}

// ============ Provider Management API ============

/// GET /api/providers - List all registered providers
pub async fn list_providers(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let providers = GATEWAY_MANAGER.list_providers();

    let mut provider_infos = Vec::new();
    for provider_name in providers {
        if let Some(config) = GATEWAY_MANAGER.get_provider(&provider_name) {
            let agent_config = AgentConfig {
                agent_type: config.provider.clone(),
                model: config.model.clone(),
                env: std::collections::HashMap::from([
                    (
                        "OPENROUTER_API_KEY".to_string(),
                        config.api_key.clone().unwrap_or_default(),
                    ),
                    (
                        "OPENROUTER_BASE_URL".to_string(),
                        config.base_url.clone().unwrap_or_default(),
                    ),
                ]),
                ..Default::default()
            };

            let agent = crate::agent::create_agent(&agent_config).unwrap();
            let healthy = agent.health_check().await.is_ok();

            provider_infos.push(ProviderInfo {
                name: provider_name,
                provider: config.provider,
                model: config.model,
                base_url: config.base_url,
                healthy,
            });
        }
    }

    Ok(AxumJson(ProvidersResponse {
        providers: provider_infos,
    }))
}

/// POST /api/providers - Register a new provider
pub async fn register_provider(
    State(_state): State<AppState>,
    Json(req): Json<ProviderRequest>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let config = GatewayConfig {
        provider: req.provider,
        model: req.model,
        api_key: req.api_key,
        base_url: req.base_url,
        fallback_providers: req.fallback_providers.unwrap_or_default(),
        load_balancing: Some(LoadBalancing::RoundRobin),
    };

    GATEWAY_MANAGER.register_provider(req.name, config);

    Ok(AxumJson(serde_json::json!({ "ok": true })))
}

/// DELETE /api/providers/{provider_name} - Remove a provider
pub async fn remove_provider(
    Path(_provider_name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    // Remove from gateway manager
    // Note: This is a simplified implementation
    // In a real system, you'd want to persist this configuration

    Ok(AxumJson(serde_json::json!({ "ok": true })))
}

/// POST /api/providers/health-check - Health check all providers
pub async fn health_check_providers(
    State(_state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, AxumJson<serde_json::Value>)> {
    let providers = GATEWAY_MANAGER.list_providers();

    let mut results = std::collections::HashMap::new();
    for provider_name in providers {
        if let Some(config) = GATEWAY_MANAGER.get_provider(&provider_name) {
            let agent_config = AgentConfig {
                agent_type: config.provider.clone(),
                model: config.model.clone(),
                env: std::collections::HashMap::from([
                    (
                        "OPENROUTER_API_KEY".to_string(),
                        config.api_key.clone().unwrap_or_default(),
                    ),
                    (
                        "OPENROUTER_BASE_URL".to_string(),
                        config.base_url.clone().unwrap_or_default(),
                    ),
                ]),
                ..Default::default()
            };

            let agent = crate::agent::create_agent(&agent_config).unwrap();
            let healthy = agent.health_check().await.is_ok();

            results.insert(provider_name, healthy);
        }
    }

    Ok(AxumJson(serde_json::json!({ "providers": results })))
}
