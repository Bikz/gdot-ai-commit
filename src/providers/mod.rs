use anyhow::Result;
use async_trait::async_trait;

use crate::config::{EffectiveConfig, OpenAiMode, ProviderKind};

mod ollama;
mod openai;

pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub max_output_tokens: u32,
    pub temperature: f32,
}

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        request: ProviderRequest,
    ) -> Result<String>;
}

pub fn build_provider(config: &EffectiveConfig) -> Result<Box<dyn Provider>> {
    match config.provider {
        ProviderKind::OpenAi => Ok(Box::new(OpenAiProvider::new(
            config.model.clone(),
            config.openai_base_url.clone(),
            config.openai_mode,
            config.timeout_secs,
            config.openai_api_key.clone(),
        )?)),
        ProviderKind::Ollama => Ok(Box::new(OllamaProvider::new(
            config.model.clone(),
            config.ollama_endpoint.clone(),
            config.timeout_secs,
        )?)),
    }
}

pub fn openai_mode_for(model: &str, mode: OpenAiMode) -> OpenAiMode {
    if mode != OpenAiMode::Auto {
        return mode;
    }

    if model.to_lowercase().starts_with("gpt-5") {
        OpenAiMode::Responses
    } else {
        OpenAiMode::Chat
    }
}
