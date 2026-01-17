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
        
        // -p/--print: 非交互模式，输出后退出
        // --dangerously-skip-permissions: 跳过权限检查（适合自动化）
        cmd.arg("-p");
        cmd.arg("--dangerously-skip-permissions");
        
        // 工作目录
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
            cmd.arg("--add-dir").arg(dir);
        }
        
        // 额外参数（如 --model, --output-format 等）
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }
        
        // prompt 作为位置参数
        cmd.arg(&prompt);
        
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
