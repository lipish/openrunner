mod anthropic;
mod claude_code;
mod codex;
mod gateway;
mod handle;
mod kimi_cli;
mod mock;
mod openai;
mod opencode;
mod openrouter;
mod traits;

pub use anthropic::AnthropicAgent;
pub use claude_code::ClaudeCodeAgent;
pub use codex::CodexAgent;
pub use gateway::{
    init_default_providers, GatewayAgent, GatewayConfig, GatewayManager, LoadBalancing,
    GATEWAY_MANAGER,
};
pub use handle::AgentHandle;
pub use kimi_cli::KimiCliAgent;
pub use mock::MockAgent;
pub use openai::OpenAIAgent;
pub use opencode::OpenCodeAgent;
pub use openrouter::OpenRouterAgent;
pub use traits::Agent;

use crate::types::AgentConfig;
use anyhow::Result;

/// 根据配置创建对应的 Agent
pub fn create_agent(config: &AgentConfig) -> Result<Box<dyn Agent>> {
    match config.agent_type.as_str() {
        "claude_code" => Ok(Box::new(ClaudeCodeAgent::new(config.clone()))),
        "codex" => Ok(Box::new(CodexAgent::new(config.clone()))),
        "opencode" => Ok(Box::new(OpenCodeAgent::new(config.clone()))),
        "kimi_cli" => Ok(Box::new(KimiCliAgent::new(config.clone()))),
        "mock" => Ok(Box::new(MockAgent::new(config.clone()))),

        // LLM Gateway providers
        "openrouter" => Ok(Box::new(OpenRouterAgent::new(config.clone()))),
        "openai" => Ok(Box::new(OpenAIAgent::new(config.clone()))),
        "anthropic" => Ok(Box::new(AnthropicAgent::new(config.clone()))),
        "gateway" => {
            // For gateway agent, we need to create a GatewayAgent
            let gateway_config = crate::agent::gateway::GatewayConfig {
                provider: config
                    .model
                    .clone()
                    .unwrap_or_else(|| "openrouter".to_string()),
                model: config.model.clone(),
                api_key: config
                    .env
                    .get("OPENROUTER_API_KEY")
                    .cloned()
                    .or_else(|| config.env.get("OPENAI_API_KEY").cloned())
                    .or_else(|| config.env.get("ANTHROPIC_API_KEY").cloned()),
                base_url: config
                    .env
                    .get("OPENROUTER_BASE_URL")
                    .cloned()
                    .or_else(|| config.env.get("OPENAI_BASE_URL").cloned())
                    .or_else(|| config.env.get("ANTHROPIC_BASE_URL").cloned()),
                fallback_providers: vec!["openai".to_string(), "anthropic".to_string()],
                load_balancing: Some(crate::agent::gateway::LoadBalancing::RoundRobin),
            };
            Ok(Box::new(crate::agent::gateway::GatewayAgent::new(
                gateway_config,
            )))
        }

        // UI-supported agents (placeholders for now)
        "droid" | "augment" | "amp" => Ok(Box::new(MockAgent::new(config.clone()))),

        _ => anyhow::bail!("Unknown agent type: {}", config.agent_type),
    }
}
