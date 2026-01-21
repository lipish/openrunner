use crate::agent::{create_agent, Agent};
use crate::types::{AgentConfig, StreamEvent};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Gateway routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub provider: String,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub fallback_providers: Vec<String>,
    pub load_balancing: Option<LoadBalancing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancing {
    RoundRobin,
    Random,
    LeastLoaded,
}

/// Gateway agent that routes to different LLM providers
pub struct GatewayAgent {
    config: GatewayConfig,
    providers: Vec<String>,
    current_provider_index: std::sync::atomic::AtomicUsize,
}

impl GatewayAgent {
    pub fn new(config: GatewayConfig) -> Self {
        let mut providers = vec![config.provider.clone()];
        providers.extend(config.fallback_providers.iter().cloned());

        Self {
            config,
            providers,
            current_provider_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    async fn get_next_provider(&self) -> String {
        match self.config.load_balancing {
            Some(LoadBalancing::RoundRobin) => {
                let idx = self
                    .current_provider_index
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.providers[idx % self.providers.len()].clone()
            }
            Some(LoadBalancing::Random) => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..self.providers.len());
                self.providers[idx].clone()
            }
            Some(LoadBalancing::LeastLoaded) => {
                // For now, just use round robin as a simple load balancing
                let idx = self
                    .current_provider_index
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.providers[idx % self.providers.len()].clone()
            }
            None => self.providers[0].clone(),
        }
    }

    async fn create_provider_agent(&self, provider: &str) -> Result<Box<dyn Agent>> {
        let agent_config = AgentConfig {
            agent_type: provider.to_string(),
            model: self.config.model.clone(),
            env: std::collections::HashMap::from([
                (
                    "OPENROUTER_API_KEY".to_string(),
                    self.config.api_key.clone().unwrap_or_default(),
                ),
                (
                    "OPENROUTER_BASE_URL".to_string(),
                    self.config.base_url.clone().unwrap_or_default(),
                ),
            ]),
            ..Default::default()
        };

        create_agent(&agent_config)
    }

    async fn try_provider(
        &self,
        provider: &str,
        prompt: String,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<()> {
        let agent = self.create_provider_agent(provider).await?;

        // Create a new channel for this attempt
        let (attempt_tx, mut attempt_rx) = mpsc::channel(100);

        // Spawn the agent
        let agent_handle = tokio::spawn(async move { agent.run(prompt, attempt_tx).await });

        // Forward events from attempt channel to main channel
        while let Some(event) = attempt_rx.recv().await {
            if tx.send(event).await.is_err() {
                // Main receiver disconnected
                break;
            }
        }

        // Wait for agent to complete
        agent_handle.await??;
        Ok(())
    }
}

#[async_trait]
impl Agent for GatewayAgent {
    fn name(&self) -> &str {
        "gateway"
    }

    async fn health_check(&self) -> Result<()> {
        for provider in &self.providers {
            let agent = self.create_provider_agent(provider).await?;
            if agent.health_check().await.is_ok() {
                return Ok(());
            }
        }
        anyhow::bail!("No healthy providers available")
    }

    async fn run(&self, prompt: String, tx: mpsc::Sender<StreamEvent>) -> Result<()> {
        let provider = self.get_next_provider().await;

        match self
            .try_provider(&provider, prompt.clone(), tx.clone())
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                // If this provider failed, try fallback providers
                for fallback_provider in &self.config.fallback_providers {
                    if let Ok(_) = self
                        .try_provider(fallback_provider, prompt.clone(), tx.clone())
                        .await
                    {
                        return Ok(());
                    }
                }
                Err(e)
            }
        }
    }
}

/// Gateway manager for handling multiple providers
pub struct GatewayManager {
    providers: std::sync::Arc<dashmap::DashMap<String, GatewayConfig>>,
}

impl GatewayManager {
    pub fn new() -> Self {
        Self {
            providers: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    pub fn register_provider(&self, name: String, config: GatewayConfig) {
        self.providers.insert(name, config);
    }

    pub fn get_provider(&self, name: &str) -> Option<GatewayConfig> {
        self.providers.get(name).map(|entry| entry.clone())
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub async fn route_request(
        &self,
        target_provider: Option<&str>,
        prompt: String,
        tx: mpsc::Sender<StreamEvent>,
    ) -> Result<()> {
        let provider_name: String = match target_provider {
            Some(name) => name.to_string(),
            None => self
                .providers
                .iter()
                .next()
                .map(|entry| entry.key().clone())
                .ok_or_else(|| anyhow::anyhow!("No providers registered"))?,
        };

        if let Some(config) = self.get_provider(&provider_name) {
            let gateway_agent = GatewayAgent::new(config);
            gateway_agent.run(prompt, tx).await
        } else {
            anyhow::bail!("Provider {} not found", provider_name)
        }
    }
}

// Global gateway instance - use once_cell instead of lazy_static
use once_cell::sync::Lazy;
pub static GATEWAY_MANAGER: Lazy<GatewayManager> = Lazy::new(GatewayManager::new);

/// Initialize default gateway providers
pub fn init_default_providers() {
    // OpenRouter provider
    let openrouter_config = GatewayConfig {
        provider: "openrouter".to_string(),
        model: Some("openai/gpt-4".to_string()),
        api_key: None, // Will be read from environment
        base_url: Some("https://openrouter.ai/api/v1".to_string()),
        fallback_providers: vec!["openai".to_string(), "anthropic".to_string()],
        load_balancing: Some(LoadBalancing::RoundRobin),
    };

    // OpenAI provider
    let openai_config = GatewayConfig {
        provider: "openai".to_string(),
        model: Some("gpt-4".to_string()),
        api_key: None,
        base_url: Some("https://api.openai.com/v1".to_string()),
        fallback_providers: vec!["openrouter".to_string()],
        load_balancing: Some(LoadBalancing::RoundRobin),
    };

    // Anthropic provider
    let anthropic_config = GatewayConfig {
        provider: "anthropic".to_string(),
        model: Some("claude-3-sonnet-20240229".to_string()),
        api_key: None,
        base_url: Some("https://api.anthropic.com/v1".to_string()),
        fallback_providers: vec!["openai".to_string()],
        load_balancing: Some(LoadBalancing::RoundRobin),
    };

    GATEWAY_MANAGER.register_provider("openrouter".to_string(), openrouter_config);
    GATEWAY_MANAGER.register_provider("openai".to_string(), openai_config);
    GATEWAY_MANAGER.register_provider("anthropic".to_string(), anthropic_config);
}
