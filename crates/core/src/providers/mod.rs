use async_trait::async_trait;

use crate::config::{EffectiveConfig, OpenAiMode, ProviderKind};
use crate::error::CoreResult;

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
    ) -> CoreResult<String>;
}

pub fn build_provider(config: &EffectiveConfig) -> CoreResult<Box<dyn Provider>> {
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
    let model = model.trim().to_lowercase();
    if model.starts_with("gpt-5") {
        return OpenAiMode::Responses;
    }

    if mode != OpenAiMode::Auto {
        return mode;
    }

    OpenAiMode::Chat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_mode_for_gpt5_forces_responses() {
        assert_eq!(
            openai_mode_for("gpt-5-nano-2025-08-07", OpenAiMode::Auto),
            OpenAiMode::Responses
        );
        assert_eq!(
            openai_mode_for("gpt-5-nano-2025-08-07", OpenAiMode::Chat),
            OpenAiMode::Responses
        );
        assert_eq!(
            openai_mode_for("gpt-5-nano-2025-08-07", OpenAiMode::Responses),
            OpenAiMode::Responses
        );
    }

    #[test]
    fn openai_mode_for_non_gpt5_respects_overrides() {
        assert_eq!(
            openai_mode_for("gpt-4o-mini", OpenAiMode::Chat),
            OpenAiMode::Chat
        );
        assert_eq!(
            openai_mode_for("gpt-4o-mini", OpenAiMode::Responses),
            OpenAiMode::Responses
        );
    }

    #[test]
    fn openai_mode_for_non_gpt5_auto_uses_chat() {
        assert_eq!(
            openai_mode_for("gpt-4o-mini", OpenAiMode::Auto),
            OpenAiMode::Chat
        );
    }
}
