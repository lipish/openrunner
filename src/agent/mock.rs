use async_trait::async_trait;
use tokio::sync::mpsc;
use anyhow::Result;
use crate::types::{AgentConfig, StreamEvent};
use super::Agent;

/// Mock Agent - 用于调试和测试，不需要真实的 API key
pub struct MockAgent {
    #[allow(dead_code)]
    config: AgentConfig,
}

impl MockAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for MockAgent {
    fn name(&self) -> &str {
        "mock"
    }

    async fn health_check(&self) -> Result<()> {
        Ok(())
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        // 模拟思考延迟（给客户端时间订阅 SSE）
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // 生成模拟回复
        let response = format!(
            "收到你的消息: \"{}\"\n\n\
            这是 Mock Agent 的模拟回复。在生产环境中，这里会调用真实的 AI Agent（如 Claude Code、Codex 等）。\n\n\
            Mock Agent 功能：\n\
            - 无需 API key\n\
            - 用于前后端联调测试\n\
            - 模拟流式输出",
            if prompt.len() > 50 { format!("{}...", &prompt[..50]) } else { prompt }
        );

        // 模拟流式输出（每个 chunk 间隔 50ms）
        for (i, chunk) in response.chars().collect::<Vec<_>>().chunks(5).enumerate() {
            if i > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            let content: String = chunk.iter().collect();
            let _ = tx.send(StreamEvent::Token { content }).await;
        }

        // 发送完成事件
        let _ = tx.send(StreamEvent::Done { 
            session_id: uuid::Uuid::new_v4()
        }).await;

        Ok(())
    }
}
