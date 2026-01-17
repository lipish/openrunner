use tokio::sync::{mpsc, oneshot};
use anyhow::Result;
use uuid::Uuid;
use crate::types::StreamEvent;
use super::Agent;

/// Agent 消息类型
pub enum AgentMessage {
    Run {
        prompt: String,
        reply: oneshot::Sender<Result<()>>,
    },
    Cancel,
}

/// Agent 句柄 - 用于与运行中的 agent 通信
pub struct AgentHandle {
    tx: mpsc::Sender<AgentMessage>,
    pub session_id: Uuid,
}

impl AgentHandle {
    /// 启动一个 agent "actor"
    pub fn spawn(
        agent: Box<dyn Agent>,
        stream_tx: mpsc::Sender<StreamEvent>,
    ) -> Self {
        let session_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::channel::<AgentMessage>(32);

        let sid = session_id;
        tokio::spawn(async move {
            tracing::info!(session_id = %sid, agent = agent.name(), "Agent started");
            
            while let Some(msg) = rx.recv().await {
                match msg {
                    AgentMessage::Run { prompt, reply } => {
                        let result = agent.run(prompt, stream_tx.clone()).await;
                        
                        // 发送完成或错误事件
                        match &result {
                            Ok(_) => {
                                let _ = stream_tx.send(StreamEvent::Done { session_id: sid }).await;
                            }
                            Err(e) => {
                                let _ = stream_tx.send(StreamEvent::Error { 
                                    message: e.to_string() 
                                }).await;
                            }
                        }
                        
                        let _ = reply.send(result);
                    }
                    AgentMessage::Cancel => {
                        tracing::info!(session_id = %sid, "Agent cancelled");
                        break;
                    }
                }
            }
            
            tracing::info!(session_id = %sid, "Agent stopped");
        });

        Self { tx, session_id }
    }

    /// 执行 prompt
    pub async fn run(&self, prompt: String) -> Result<()> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(AgentMessage::Run { prompt, reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("Agent channel closed"))?;
        reply_rx.await?
    }

    /// 取消执行
    pub async fn cancel(&self) -> Result<()> {
        self.tx
            .send(AgentMessage::Cancel)
            .await
            .map_err(|_| anyhow::anyhow!("Agent channel closed"))
    }
}
