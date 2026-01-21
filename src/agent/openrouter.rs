use super::Agent;
use crate::types::{AgentConfig, StreamEvent};
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// OpenRouter API client for LLM gateway functionality
pub struct OpenRouterAgent {
    config: AgentConfig,
    client: Client,
}

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
    stream: Option<bool>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenRouterStreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<Delta>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
}

impl OpenRouterAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    async fn get_api_key(&self) -> Result<String> {
        // Try to get API key from config env, then from environment variables
        if let Some(api_key) = self.config.env.get("OPENROUTER_API_KEY") {
            return Ok(api_key.clone());
        }

        if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
            return Ok(api_key);
        }

        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            return Ok(api_key);
        }

        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            return Ok(api_key);
        }

        anyhow::bail!(
            "No API key found. Set OPENROUTER_API_KEY, OPENAI_API_KEY, or ANTHROPIC_API_KEY"
        )
    }

    async fn get_base_url(&self) -> String {
        self.config
            .env
            .get("OPENROUTER_BASE_URL")
            .cloned()
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string())
    }

    async fn get_model_name(&self) -> String {
        self.config
            .model
            .clone()
            .unwrap_or_else(|| "openai/gpt-4".to_string())
    }
}

#[async_trait]
impl Agent for OpenRouterAgent {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn health_check(&self) -> Result<()> {
        let api_key = self.get_api_key().await?;
        let base_url = self.get_base_url().await;
        let model_name = self.get_model_name().await;

        let client = reqwest::Client::new();
        let response = client
            .get(&format!("{}/models", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/openrunner")
            .header("X-Title", "OpenRunner")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            anyhow::bail!("OpenRouter API health check failed: {}", response.status())
        }
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let api_key = self.get_api_key().await?;
        let base_url = self.get_base_url().await;
        let model_name = self.get_model_name().await;

        // Build request
        let request = OpenRouterRequest {
            model: model_name,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            stream: Some(true),
            temperature: None,
            max_tokens: None,
        };

        // Send streaming request
        let response = self
            .client
            .post(&format!("{}/chat/completions", base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("HTTP-Referer", "https://github.com/openrunner")
            .header("X-Title", "OpenRunner")
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenRouter API error: {}", error_text));
        }

        // Process streaming response
        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        let mut full_response = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    let chunk_str = String::from_utf8_lossy(&chunk);

                    // Parse SSE format
                    for line in chunk_str.lines() {
                        if line.starts_with("data: ") {
                            let data = &line[6..];

                            if data == "[DONE]" {
                                // Send final completion token
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
                                serde_json::from_str::<OpenRouterStreamChunk>(data)
                            {
                                for choice in stream_data.choices {
                                    if let Some(delta) = choice.delta {
                                        if let Some(content) = delta.content {
                                            full_response.push_str(&content);
                                            let _ = tx.send(StreamEvent::Token { content }).await;
                                        }
                                    }

                                    if choice.finish_reason.is_some() {
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

        // Send completion even if stream ended unexpectedly
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
