use super::Agent;
use crate::types::{AgentConfig, StreamEvent};
use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

/// OpenCode Agent - 调用 opencode CLI
/// https://github.com/opencode-ai/opencode
///
/// 安装: npm i -g opencode-ai
/// 或: curl -fsSL https://opencode.ai/install | bash
///
/// 支持的环境变量 (前端可配置):
/// - OPENCODE_BASE_URL: API 基础 URL
/// - OPENCODE_API_KEY: API 密钥
/// - OPENCODE_PROVIDER: Provider 名称 (默认: custom)
pub struct OpenCodeAgent {
    config: AgentConfig,
}

impl OpenCodeAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    /// 构建 opencode 命令行参数
    fn build_args(&self, prompt: &str) -> Vec<String> {
        let mut args = vec!["run".to_string(), "--format=json".to_string()];

        let has_model_arg = self
            .config
            .extra_args
            .iter()
            .any(|a| a == "--model" || a == "-m");

        // 额外参数（如 --model, --provider 等）
        for arg in &self.config.extra_args {
            args.push(arg.clone());
        }

        if let Some(ref model) = self.config.model {
            if !has_model_arg {
                args.push("--model".to_string());
                args.push(model.clone());
            }
        }

        // Prompt 作为最后一个位置参数
        args.push(prompt.to_string());

        args
    }

    /// 根据环境变量动态生成 opencode 配置
    /// 返回 (临时配置目录路径, 需要使用的 model 名称)
    fn create_dynamic_config(&self) -> Result<Option<(PathBuf, String)>> {
        let base_url = self.config.env.get("OPENCODE_BASE_URL");
        let api_key = self.config.env.get("OPENCODE_API_KEY");

        // 如果没有提供动态配置相关的环境变量，返回 None
        if base_url.is_none() && api_key.is_none() {
            return Ok(None);
        }

        let base_url = base_url
            .map(|s| s.as_str())
            .unwrap_or("https://api.openai.com/v1");
        let api_key = api_key.map(|s| s.as_str()).unwrap_or("");
        let provider_name = self
            .config
            .env
            .get("OPENCODE_PROVIDER")
            .map(|s| s.as_str())
            .unwrap_or("custom");

        // 从 model 配置中提取模型名称 (如果已经是 provider/model 格式则提取 model 部分)
        let model_name = self
            .config
            .model
            .as_ref()
            .map(|m| {
                if m.contains('/') {
                    m.split('/').last().unwrap_or(m).to_string()
                } else {
                    m.clone()
                }
            })
            .unwrap_or_else(|| "gpt-4".to_string());

        // 创建临时配置目录
        let temp_dir = std::env::temp_dir().join(format!("opencode-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir)?;

        let config_dir = temp_dir.join("opencode");
        std::fs::create_dir_all(&config_dir)?;

        // 生成 opencode.json 配置
        let config = serde_json::json!({
            "$schema": "https://opencode.ai/config.json",
            "provider": {
                provider_name: {
                    "npm": "@ai-sdk/openai-compatible",
                    "options": {
                        "baseURL": base_url,
                        "apiKey": api_key
                    },
                    "models": {
                        &model_name: {
                            "name": &model_name
                        }
                    }
                }
            }
        });

        let config_path = config_dir.join("opencode.json");
        std::fs::write(&config_path, serde_json::to_string_pretty(&config)?)?;

        tracing::info!("Created dynamic opencode config at: {:?}", config_path);

        // 返回配置目录和完整的 model 名称 (provider/model)
        let full_model = format!("{}/{}", provider_name, model_name);
        Ok(Some((temp_dir, full_model)))
    }

    /// 清理临时配置目录
    fn cleanup_config(&self, config_dir: &PathBuf) {
        if let Err(e) = std::fs::remove_dir_all(config_dir) {
            tracing::warn!("Failed to cleanup temp config dir {:?}: {}", config_dir, e);
        }
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
        // 调试日志：显示环境变量
        tracing::info!(
            "OpenCode run - env vars: {:?}, model: {:?}, extra_args: {:?}",
            self.config.env.keys().collect::<Vec<_>>(),
            self.config.model,
            self.config.extra_args
        );

        // 尝试创建动态配置
        let dynamic_config = self.create_dynamic_config()?;
        let temp_dir = dynamic_config.as_ref().map(|(dir, _)| dir.clone());

        // 如果有动态配置，使用配置中的 model 名称
        let effective_model = dynamic_config
            .as_ref()
            .map(|(_, model)| model.clone())
            .or_else(|| self.config.model.clone());

        // 构建命令行参数
        let mut args = vec!["run".to_string(), "--format=json".to_string()];
        let has_model_arg = self
            .config
            .extra_args
            .iter()
            .any(|a| a == "--model" || a == "-m");

        for arg in &self.config.extra_args {
            args.push(arg.clone());
        }

        if let Some(ref model) = effective_model {
            if !has_model_arg {
                args.push("--model".to_string());
                args.push(model.clone());
            }
        }
        args.push(prompt.clone());

        // 构建 expect 脚本来提供 PTY
        let opencode_cmd = format!(
            "opencode {}",
            args.iter()
                .map(|a| shell_escape::escape(std::borrow::Cow::Borrowed(a)).to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let expect_script = format!(
            r#"
set timeout 600
spawn -noecho {cmd}
expect {{
    timeout {{ puts "OPENCODE_TIMEOUT"; exit 1 }}
    eof {{ }}
}}
"#,
            cmd = opencode_cmd
        );

        let mut cmd = tokio::process::Command::new("expect");
        cmd.arg("-c");
        cmd.arg(&expect_script);

        // 工作目录
        if let Some(ref dir) = self.config.working_dir {
            cmd.current_dir(dir);
        }

        // 如果有动态配置，设置 XDG_CONFIG_HOME 环境变量
        if let Some(ref dir) = temp_dir {
            cmd.env("XDG_CONFIG_HOME", dir);
            tracing::info!("Using dynamic config from: {:?}", dir);
        }

        // 传递其他环境变量（排除我们内部使用的）
        for (k, v) in &self.config.env {
            if k != "OPENCODE_BASE_URL" && k != "OPENCODE_API_KEY" && k != "OPENCODE_PROVIDER" {
                cmd.env(k, v);
            }
        }

        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
        let mut stderr_handle = child.stderr.take();

        let mut reader = BufReader::new(stdout).lines();
        let mut result: Result<()> = Ok(());

        while let Some(line) = reader.next_line().await? {
            tracing::debug!("opencode output line: {}", line);

            // 跳过 expect 的 spawn 行
            if line.starts_with("spawn ") {
                continue;
            }

            // 解析 JSON 事件格式
            if line.starts_with('{') {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                    let event_type = event
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or("unknown");
                    tracing::debug!("opencode event type: {}", event_type);

                    match event_type {
                        "text" => {
                            // 文本输出事件 - opencode 格式: {"type":"text","part":{"text":"..."}}
                            let text = event
                                .get("part")
                                .and_then(|p| p.get("text"))
                                .and_then(|t| t.as_str())
                                .or_else(|| event.get("content").and_then(|c| c.as_str()));

                            if let Some(content) = text {
                                if tx
                                    .send(StreamEvent::Token {
                                        content: content.to_string(),
                                    })
                                    .await
                                    .is_err()
                                {
                                    let _ = child.kill().await;
                                    break;
                                }
                            }
                        }
                        "error" => {
                            // 错误事件
                            let error_msg = event
                                .get("error")
                                .and_then(|e| e.get("data"))
                                .and_then(|d| d.get("message"))
                                .and_then(|m| m.as_str())
                                .or_else(|| {
                                    event
                                        .get("error")
                                        .and_then(|e| e.get("message"))
                                        .and_then(|m| m.as_str())
                                })
                                .or_else(|| event.get("message").and_then(|m| m.as_str()))
                                .unwrap_or("Unknown error");
                            tracing::error!("opencode error: {}", error_msg);
                            result = Err(anyhow::anyhow!("opencode error: {}", error_msg));
                            // 发送错误消息给前端
                            let _ = tx
                                .send(StreamEvent::Token {
                                    content: format!("Error: {}", error_msg),
                                })
                                .await;
                            break;
                        }
                        // 忽略 step_start, step_finish 等内部事件
                        "step_start" | "step_finish" | "tool_start" | "tool_finish" => {
                            // 这些是内部事件，不需要发送给前端
                            continue;
                        }
                        _ => {
                            // 其他未知事件类型，记录但不发送
                            tracing::debug!("opencode unknown event: {}", event_type);
                        }
                    }
                }
            } else if !line.trim().is_empty() && !line.contains("OPENCODE_TIMEOUT") {
                // 非 JSON 行，可能是错误信息
                tracing::debug!("opencode non-JSON line: {}", line);
                // 只发送看起来像错误的内容
                if line.contains("Error") || line.contains("error") {
                    let _ = tx
                        .send(StreamEvent::Token {
                            content: format!("{}\n", line),
                        })
                        .await;
                }
            }
        }

        let status = child.wait().await?;

        // 清理临时配置目录
        if let Some(ref dir) = temp_dir {
            self.cleanup_config(dir);
        }

        if result.is_err() {
            return result;
        }

        if !status.success() {
            let mut stderr = String::new();
            if let Some(mut err) = stderr_handle.take() {
                let _ = err.read_to_string(&mut stderr).await;
            }
            if stderr.trim().is_empty() {
                anyhow::bail!("opencode exited with status: {}", status);
            }
            anyhow::bail!("opencode failed: {}", stderr.trim());
        }

        Ok(())
    }
}
