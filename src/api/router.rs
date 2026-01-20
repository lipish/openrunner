use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::run::{RunStore, RunManager};
use crate::storage::Db;
use super::handlers;

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

        // 状态
        .with_state(state)
        
        // 中间件
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
