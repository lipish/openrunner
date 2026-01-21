use super::Agent;
use crate::types::{AgentConfig, StreamEvent};
use anyhow::Result;
use async_trait::async_trait;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tracing::info;

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

        let mut args: Vec<String> = Vec::new();

        if let Some(ref model) = self.config.model {
            args.push("--model".to_string());
            args.push(model.to_string());
        }

        if let Some(ref dir) = self.config.working_dir {
            args.push("--work-dir".to_string());
            args.push(dir.to_string());
            cmd.current_dir(dir);
        }

        args.extend(self.config.extra_args.iter().cloned());
        args.push("--print".to_string());
        args.push("--command".to_string());
        args.push(prompt.clone());

        let mut redacted = args.clone();
        for item in &mut redacted {
            let lower = item.to_lowercase();
            if lower.contains("api_key") || lower.contains("sk-") {
                *item = "[redacted]".to_string();
            }
            if item == &prompt {
                *item = "<prompt>".to_string();
            }
        }
        let env_keys: Vec<&String> = self.config.env.keys().collect();
        info!("kimi args: {:?}, env keys: {:?}", redacted, env_keys);

        for arg in &args {
            cmd.arg(arg);
        }

        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }

        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let mut stderr_handle = child.stderr.take();
        let mut reader = BufReader::new(stdout).lines();
        let mut last_output: Option<String> = None;

        while let Some(line) = reader.next_line().await? {
            if !line.trim().is_empty() {
                last_output = Some(line.clone());
            }
            if tx
                .send(StreamEvent::Token {
                    content: format!("{}\n", line),
                })
                .await
                .is_err()
            {
                child.kill().await?;
                break;
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut err) = stderr_handle.take() {
                let _ = err.read_to_string(&mut stderr).await;
            }
            if stderr.trim().is_empty() {
                if let Some(line) = last_output {
                    anyhow::bail!("kimi failed: {}", line.trim());
                }
                anyhow::bail!("kimi exited with status: {}", status);
            }
            anyhow::bail!("kimi failed: {}", stderr.trim());
        }

        Ok(())
    }
}
