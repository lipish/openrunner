use super::Agent;
use crate::types::{AgentConfig, StreamEvent};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Direct Anthropic Claude API integration
pub struct AnthropicAgent {
    config: AgentConfig,
    client: Client,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    stream: Option<bool>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicStreamChunk {
    #[serde(rename = "type")]
    chunk_type: String,
    delta: Option<Delta>,
    index: Option<u32>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    text: Option<String>,
}

impl AnthropicAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    async fn get_api_key(&self) -> Result<String> {
        if let Some(api_key) = self.config.env.get("ANTHROPIC_API_KEY") {
            return Ok(api_key.clone());
        }

        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            return Ok(api_key);
        }

        anyhow::bail!("No Anthropic API key found. Set ANTHROPIC_API_KEY")
    }

    async fn get_base_url(&self) -> String {
        self.config
            .env
            .get("ANTHROPIC_BASE_URL")
            .cloned()
            .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string())
    }

    async fn get_model_name(&self) -> String {
        self.config
            .model
            .clone()
            .unwrap_or_else(|| "claude-3-sonnet-20240229".to_string())
    }
}

#[async_trait]
impl Agent for AnthropicAgent {
    fn name(&self) -> &str {
        "anthropic"
    }

    async fn health_check(&self) -> Result<()> {
        let api_key = self.get_api_key().await?;
        let base_url = self.get_base_url().await;

        let response = self
            .client
            .get(&format!("{}/models", base_url))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("Anthropic API health check failed: {}", response.status())
        }
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let api_key = self.get_api_key().await?;
        let base_url = self.get_base_url().await;
        let model_name = self.get_model_name().await;

        // Build request - Anthropic uses different message format
        let request = AnthropicRequest {
            model: model_name,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: Some(true),
            temperature: None,
            max_tokens: None,
            top_p: None,
        };

        // Send streaming request
        let response = self
            .client
            .post(&format!("{}/messages", base_url))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        // Process streaming response
        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        let mut full_response = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8_lossy(&chunk);

                    // Parse SSE format for Anthropic
                    for line in chunk_str.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];

                            if data == "[DONE]" {
                                if !full_response.is_empty() {
                                    let _ = tx
                                        .send(StreamEvent::Token {
                                            content: full_response.clone(),
                                        })
                                        .await;
                                }
                                let _ = tx
                                    .send(StreamEvent::Done {
                                        session_id: uuid::Uuid::new_v4(),
                                    })
                                    .await;
                                return Ok(());
                            }

                            if let Ok(stream_data) =
                                serde_json::from_str::<AnthropicStreamChunk>(data)
                            {
                                if stream_data.chunk_type == "content_block_delta" {
                                    if let Some(delta) = stream_data.delta {
                                        if let Some(text) = delta.text {
                                            full_response.push_str(&text);
                                            let _ =
                                                tx.send(StreamEvent::Token { content: text }).await;
                                        }
                                    }
                                }

                                if stream_data.finish_reason.is_some() {
                                    let _ = tx
                                        .send(StreamEvent::Done {
                                            session_id: uuid::Uuid::new_v4(),
                                        })
                                        .await;
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = tx
                        .send(StreamEvent::Error {
                            message: format!("Stream error: {}", e),
                        })
                        .await;
                    return Err(anyhow::anyhow!("Stream error: {}", e));
                }
            }
        }

        if !full_response.is_empty() {
            let _ = tx
                .send(StreamEvent::Token {
                    content: full_response,
                })
                .await;
        }
        let _ = tx
            .send(StreamEvent::Done {
                session_id: uuid::Uuid::new_v4(),
            })
            .await;

        Ok(())
    }
}
