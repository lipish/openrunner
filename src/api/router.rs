use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::handlers;
use super::openrouter;
use crate::run::{RunManager, RunStore};
use crate::storage::Db;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub run_manager: RunManager,
    pub db: Db,
}

impl AppState {
    pub async fn new() -> Self {
        let store = RunStore::new();
        let run_manager = RunManager::new(store);
        let db = Db::new("data/openrunner.db")
            .await
            .expect("Failed to initialize database");
        Self { run_manager, db }
    }
}

/// 创建 API 路由
pub async fn create_router() -> Router {
    let state = AppState::new().await;
    create_router_with_state(state)
}

/// 使用指定状态创建路由
pub fn create_router_with_state(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // 健康检查
        .route("/health", get(handlers::health))
        .route("/health/agents", get(handlers::health_agents))
        // Agent 列表
        .route("/agents", get(handlers::list_agents))
        // Auth API
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/register", post(handlers::register))
        // Runs API
        .route("/api/runs", post(handlers::create_run))
        .route("/api/runs/:run_id/events", get(handlers::run_events))
        // Chat API (fallback)
        .route("/api/chat", post(handlers::chat))
        // Sessions API
        .route("/api/sessions", get(handlers::list_sessions))
        .route("/api/sessions", post(handlers::save_sessions))
        // Agent Defaults API
        .route("/api/agent-defaults", get(handlers::get_agent_defaults))
        .route("/api/agent-defaults", post(handlers::set_agent_default))
        // Projects API
        .route("/api/projects", get(handlers::list_projects))
        .route("/api/projects", post(handlers::create_project))
        .route("/api/projects/:project_id", delete(handlers::delete_project))
        // OpenRouter-compatible API endpoints
        .route(
            "/v1/chat/completions",
            post(openrouter::openrouter_chat_completions),
        )
        .route(
            "/v1/chat/completions/stream",
            post(openrouter::openrouter_chat_completions_stream),
        )
        .route("/v1/models", get(openrouter::openrouter_models))
        .route(
            "/v1/models/:model_id",
            get(openrouter::openrouter_model_details),
        )
        // Provider management API
        .route("/api/providers", get(openrouter::list_providers))
        .route("/api/providers", post(openrouter::register_provider))
        .route(
            "/api/providers/:provider_name",
            axum::routing::delete(openrouter::remove_provider),
        )
        .route(
            "/api/providers/health-check",
            post(openrouter::health_check_providers),
        )
        // 状态
        .with_state(state)
        // 中间件
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
