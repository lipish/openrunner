use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;
use crate::types::{AgentConfig, StreamEvent};
use super::Agent;

/// Claude Code Agent - 调用 claude CLI
pub struct ClaudeCodeAgent {
    config: AgentConfig,
}

impl ClaudeCodeAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for ClaudeCodeAgent {
    fn name(&self) -> &str {
        "claude_code"
    }

    async fn health_check(&self) -> Result<()> {
        let output = tokio::process::Command::new("claude")
            .arg("--version")
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("claude CLI not available");
        }
        Ok(())
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("claude");
        
        // 基础参数
        cmd.arg("--print");  // 非交互模式
        cmd.arg("--prompt").arg(&prompt);
        
        // 工作目录
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }
        
        // 额外参数
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }
        
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        
        let mut reader = BufReader::new(stdout).lines();
        
        // 流式读取输出
        while let Some(line) = reader.next_line().await? {
            if tx.send(StreamEvent::Token { content: format!("{}\n", line) }).await.is_err() {
                // 接收方已关闭，终止进程
                child.kill().await?;
                break;
            }
        }
        
        let status = child.wait().await?;
        if !status.success() {
            anyhow::bail!("claude exited with status: {}", status);
        }
        
        Ok(())
    }
}
