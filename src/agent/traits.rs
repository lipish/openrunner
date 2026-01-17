use async_trait::async_trait;
use tokio::sync::mpsc;
use anyhow::Result;
use crate::types::StreamEvent;

/// Agent trait - 所有 agent 实现的核心接口
#[async_trait]
pub trait Agent: Send + Sync + 'static {
    /// 执行 agent，流式输出到 tx
    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()>;
    
    /// Agent 名称
    fn name(&self) -> &str;
    
    /// 检查 agent 是否可用（CLI 是否安装等）
    async fn health_check(&self) -> Result<()> {
        Ok(())
    }
}
