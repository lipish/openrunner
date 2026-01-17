use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "openrunner=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建路由
    let app = openrunner::create_router();

    // 绑定地址
    let addr = std::env::var("OPENRUNNER_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("OpenRunner listening on {}", addr);
    tracing::info!("API endpoints:");
    tracing::info!("  GET  /health     - Health check");
    tracing::info!("  GET  /agents     - List available agents");
    tracing::info!("  POST /run        - Run agent (SSE streaming)");
    tracing::info!("  POST /run/sync   - Run agent (wait for completion)");

    // 启动服务
    axum::serve(listener, app).await?;

    Ok(())
}
