use dashmap::DashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::RunEvent;

/// Run 状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Run 信息
#[derive(Debug, Clone)]
pub struct Run {
    pub id: String,
    pub session_id: Option<String>,
    pub user_id: String,
    pub status: RunStatus,
    pub input_text: String,
    pub output: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// 事件发送器（用于广播给订阅者）
    pub event_tx: Option<mpsc::Sender<RunEvent>>,
}

/// Run 存储
#[derive(Clone)]
pub struct RunStore {
    runs: Arc<DashMap<String, Run>>,
}

impl RunStore {
    pub fn new() -> Self {
        Self {
            runs: Arc::new(DashMap::new()),
        }
    }

    /// 创建新 Run
    pub fn create(&self, run_id: String, user_id: String, session_id: Option<String>, input_text: String) -> Run {
        let now = Utc::now();
        let run = Run {
            id: run_id.clone(),
            session_id,
            user_id,
            status: RunStatus::Pending,
            input_text,
            output: String::new(),
            error: None,
            created_at: now,
            updated_at: now,
            event_tx: None,
        };
        self.runs.insert(run_id, run.clone());
        run
    }

    /// 获取 Run
    pub fn get(&self, run_id: &str) -> Option<Run> {
        self.runs.get(run_id).map(|r| r.clone())
    }

    /// 更新 Run 状态
    pub fn update_status(&self, run_id: &str, status: RunStatus) {
        if let Some(mut run) = self.runs.get_mut(run_id) {
            run.status = status;
            run.updated_at = Utc::now();
        }
    }

    /// 追加输出
    pub fn append_output(&self, run_id: &str, content: &str) {
        if let Some(mut run) = self.runs.get_mut(run_id) {
            run.output.push_str(content);
            run.updated_at = Utc::now();
        }
    }

    /// 设置错误
    pub fn set_error(&self, run_id: &str, error: String) {
        if let Some(mut run) = self.runs.get_mut(run_id) {
            run.error = Some(error);
            run.status = RunStatus::Failed;
            run.updated_at = Utc::now();
        }
    }

    /// 设置事件发送器
    pub fn set_event_tx(&self, run_id: &str, tx: mpsc::Sender<RunEvent>) {
        if let Some(mut run) = self.runs.get_mut(run_id) {
            run.event_tx = Some(tx);
        }
    }

    /// 获取事件发送器
    pub fn get_event_tx(&self, run_id: &str) -> Option<mpsc::Sender<RunEvent>> {
        self.runs.get(run_id).and_then(|r| r.event_tx.clone())
    }

    /// 删除 Run
    pub fn remove(&self, run_id: &str) -> Option<Run> {
        self.runs.remove(run_id).map(|(_, r)| r)
    }

    /// 列出用户的 Runs
    pub fn list_by_user(&self, user_id: &str) -> Vec<Run> {
        self.runs
            .iter()
            .filter(|r| r.user_id == user_id)
            .map(|r| r.clone())
            .collect()
    }

    /// 清理过期的 Runs（超过指定时间的已完成/失败 runs）
    pub fn cleanup_expired(&self, max_age_secs: i64) {
        let now = Utc::now();
        let to_remove: Vec<String> = self.runs
            .iter()
            .filter(|r| {
                matches!(r.status, RunStatus::Completed | RunStatus::Failed | RunStatus::Cancelled)
                    && (now - r.updated_at).num_seconds() > max_age_secs
            })
            .map(|r| r.id.clone())
            .collect();

        for id in to_remove {
            self.runs.remove(&id);
        }
    }
}

impl Default for RunStore {
    fn default() -> Self {
        Self::new()
    }
}
