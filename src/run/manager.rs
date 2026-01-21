use chrono::Utc;
use tokio::sync::mpsc;
use uuid::Uuid;

use super::{
    CompletedMessage, MessageDelta, Run, RunCompleted, RunEvent, RunFailed, RunStatus, RunStore,
};
use crate::agent::{create_agent, AgentHandle};
use crate::types::{AgentConfig, StreamEvent};

/// Run 管理器 - 负责创建和管理 agent 执行
#[derive(Clone)]
pub struct RunManager {
    store: RunStore,
}

impl RunManager {
    pub fn new(store: RunStore) -> Self {
        Self { store }
    }

    /// 创建新 Run
    pub fn create_run(
        &self,
        user_id: &str,
        session_id: Option<String>,
        input_text: &str,
    ) -> String {
        let run_id = format!(
            "run_{}",
            Uuid::new_v4().to_string().replace("-", "")[..12].to_string()
        );
        self.store.create(
            run_id.clone(),
            user_id.to_string(),
            session_id,
            input_text.to_string(),
        );
        run_id
    }

    /// 启动 Run 执行
    pub async fn start_run(&self, run_id: &str, config: AgentConfig) -> anyhow::Result<()> {
        let run = self
            .store
            .get(run_id)
            .ok_or_else(|| anyhow::anyhow!("Run not found: {}", run_id))?;

        // 创建 agent
        let agent = create_agent(&config)?;

        // 创建内部 channel 接收 agent 事件
        let (agent_tx, mut agent_rx) = mpsc::channel::<StreamEvent>(100);

        // 启动 agent
        let handle = AgentHandle::spawn(agent, agent_tx);

        // 更新状态为运行中
        self.store.update_status(run_id, RunStatus::Running);

        let store = self.store.clone();
        let rid = run_id.to_string();
        let prompt = run.input_text.clone();

        // 启动事件转发任务
        tokio::spawn(async move {
            // 在单独的 task 中启动 agent 执行
            let run_task = tokio::spawn(async move { handle.run(prompt).await });

            // 并发处理：转发事件
            let mut output = String::new();

            loop {
                match agent_rx.recv().await {
                    Some(StreamEvent::Token { content }) => {
                        output.push_str(&content);
                        store.append_output(&rid, &content);

                        // 转发给订阅者
                        if let Some(tx) = store.get_event_tx(&rid) {
                            let _ = tx
                                .send(RunEvent::MessageDelta(MessageDelta { delta: content }))
                                .await;
                        }
                    }
                    Some(StreamEvent::Done { .. }) => {
                        store.update_status(&rid, RunStatus::Completed);

                        if let Some(tx) = store.get_event_tx(&rid) {
                            let _ = tx
                                .send(RunEvent::RunCompleted(RunCompleted {
                                    message: CompletedMessage {
                                        role: "assistant".to_string(),
                                        content: output.clone(),
                                        timestamp: Utc::now(),
                                    },
                                }))
                                .await;
                        }
                        break;
                    }
                    Some(StreamEvent::Error { message }) => {
                        store.set_error(&rid, message.clone());

                        if let Some(tx) = store.get_event_tx(&rid) {
                            let _ = tx
                                .send(RunEvent::RunFailed(RunFailed { error: message }))
                                .await;
                        }
                        break;
                    }
                    None => break,
                }
            }

            // 等待 agent 任务完成
            let _ = run_task.await;
        });

        Ok(())
    }

    /// 订阅 Run 事件
    pub fn subscribe(&self, run_id: &str) -> Option<mpsc::Receiver<RunEvent>> {
        let (tx, rx) = mpsc::channel::<RunEvent>(100);

        if self.store.get(run_id).is_some() {
            self.store.set_event_tx(run_id, tx);
            Some(rx)
        } else {
            None
        }
    }

    /// 取消 Run
    pub fn cancel_run(&self, run_id: &str) -> bool {
        if let Some(_run) = self.store.get(run_id) {
            self.store.update_status(run_id, RunStatus::Cancelled);
            // TODO: 实际终止 agent 进程
            true
        } else {
            false
        }
    }

    /// 获取 Run 信息
    pub fn get_run(&self, run_id: &str) -> Option<Run> {
        self.store.get(run_id)
    }

    /// 获取 store 引用
    pub fn store(&self) -> &RunStore {
        &self.store
    }
}
