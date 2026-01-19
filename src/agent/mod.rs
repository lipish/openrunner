mod traits;
mod handle;
mod claude_code;
mod codex;
mod opencode;
mod kimi_cli;
mod mock;

pub use traits::Agent;
pub use handle::AgentHandle;
pub use claude_code::ClaudeCodeAgent;
pub use codex::CodexAgent;
pub use opencode::OpenCodeAgent;
pub use kimi_cli::KimiCliAgent;
pub use mock::MockAgent;

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

        // UI-supported agents (placeholders for now)
        "droid" | "augment" | "amp" => Ok(Box::new(MockAgent::new(config.clone()))),

        _ => anyhow::bail!("Unknown agent type: {}", config.agent_type),
    }
}
