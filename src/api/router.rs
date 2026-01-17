use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use super::handlers;

/// 创建 API 路由
pub fn create_router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // 健康检查
        .route("/health", get(handlers::health))
        .route("/health/agents", get(handlers::health_agents))
        // Agent 相关
        .route("/agents", get(handlers::list_agents))
        .route("/run", post(handlers::run_agent))           // 流式
        .route("/run/sync", post(handlers::run_agent_sync)) // 同步
        // 中间件
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}
