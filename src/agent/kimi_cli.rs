use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use anyhow::Result;
use crate::types::{AgentConfig, StreamEvent};
use super::Agent;

/// Kimi CLI Agent - 调用 kimi CLI
pub struct KimiCliAgent {
    config: AgentConfig,
}

impl KimiCliAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Agent for KimiCliAgent {
    fn name(&self) -> &str {
        "kimi_cli"
    }

    async fn health_check(&self) -> Result<()> {
        let output = tokio::process::Command::new("kimi")
            .arg("--version")
            .output()
            .await?;

        if !output.status.success() {
            anyhow::bail!("kimi CLI not available. Install: https://moonshotai.github.io/kimi-cli/en/guides/getting-started.html");
        }
        Ok(())
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let mut cmd = tokio::process::Command::new("kimi");

        // 非交互模式：使用 --prompt/-p 直接传入 prompt
        cmd.arg("--prompt").arg(&prompt);

        // 选择模型（可选）
        if let Some(ref model) = self.config.model {
            cmd.arg("--model").arg(model);
        }

        // 工作目录
        if let Some(ref dir) = self.config.working_dir {
            cmd.arg("--work-dir").arg(dir);
            cmd.current_dir(dir);
        }

        // 环境变量
        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }

        // 额外参数（如 --agent, --config-file 等）
        for arg in &self.config.extra_args {
            cmd.arg(arg);
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
            anyhow::bail!("kimi exited with status: {}", status);
        }

        Ok(())
    }
}
