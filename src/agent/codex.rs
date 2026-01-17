use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;
use crate::types::{AgentConfig, StreamEvent};
use super::Agent;

/// Codex Agent - 调用 OpenAI Codex CLI
/// https://github.com/openai/codex
/// 
/// 安装: npm install -g @openai/codex
pub struct CodexAgent {
    config: AgentConfig,
}

impl CodexAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for CodexAgent {
    fn name(&self) -> &str {
        "codex"
    }

    async fn health_check(&self) -> Result<()> {
        let output = tokio::process::Command::new("codex")
            .arg("--version")
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("codex CLI not available. Install: npm install -g @openai/codex");
        }
        Ok(())
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("codex");
        
        // Codex CLI 参数
        // --quiet: 减少冗余输出
        // --approval-mode full-auto: 自动批准所有操作（非交互）
        cmd.arg("--quiet");
        cmd.arg("--approval-mode").arg("full-auto");
        
        // 额外参数（用户可通过 extra_args 传入如 --model, --json 等）
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }
        
        // prompt 作为最后一个位置参数
        cmd.arg(&prompt);
        
        // 工作目录
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }
        
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        
        let mut reader = BufReader::new(stdout).lines();
        
        while let Some(line) = reader.next_line().await? {
            if tx.send(StreamEvent::Token { content: format!("{}\n", line) }).await.is_err() {
                child.kill().await?;
                break;
            }
        }
        
        let status = child.wait().await?;
        if !status.success() {
            // 读取 stderr 获取错误信息
            anyhow::bail!("codex exited with status: {}", status);
        }
        
        Ok(())
    }
}
