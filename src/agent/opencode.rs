use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::io::{AsyncBufReadExt, BufReader};
use anyhow::Result;
use crate::types::{AgentConfig, StreamEvent};
use super::Agent;

/// OpenCode Agent - 调用 opencode CLI
/// https://github.com/opencode-ai/opencode
/// 
/// 安装: go install github.com/opencode-ai/opencode@latest
/// 或: brew install opencode-ai/tap/opencode
pub struct OpenCodeAgent {
    config: AgentConfig,
}

impl OpenCodeAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for OpenCodeAgent {
    fn name(&self) -> &str {
        "opencode"
    }

    async fn health_check(&self) -> Result<()> {
        let output = tokio::process::Command::new("opencode")
            .arg("--version")
            .output()
            .await?;
        
        if !output.status.success() {
            anyhow::bail!("opencode CLI not available. Install: go install github.com/opencode-ai/opencode@latest");
        }
        Ok(())
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("opencode");
        
        // OpenCode 非交互模式
        // -p: 直接传入 prompt（非交互）
        // -q: quiet 模式，减少输出
        cmd.arg("-p").arg(&prompt);
        
        // 额外参数（如 --model, --provider 等）
        for arg in &self.config.extra_args {
            cmd.arg(arg);
        }
        
        // 工作目录
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }
        
        // 禁用 TTY，强制非交互
        cmd.env("TERM", "dumb");
        cmd.stdin(std::process::Stdio::null());
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
            anyhow::bail!("opencode exited with status: {}", status);
        }
        
        Ok(())
    }
}
