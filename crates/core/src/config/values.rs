use serde::{Deserialize, Serialize};

use crate::error::CoreResult;

use super::types::{OpenAiMode, ProviderKind, StageMode};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Config {
    pub provider: Option<ProviderKind>,
    pub model: Option<String>,
    pub openai_mode: Option<OpenAiMode>,
    pub openai_base_url: Option<String>,
    pub openai_api_key: Option<String>,
    pub ollama_endpoint: Option<String>,
    pub conventional: Option<bool>,
    pub one_line: Option<bool>,
    pub emoji: Option<bool>,
    pub lang: Option<String>,
    pub push: Option<bool>,
    pub timeout_secs: Option<u64>,
    pub max_input_tokens: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub max_file_bytes: Option<u64>,
    pub max_file_lines: Option<u32>,
    pub summary_concurrency: Option<u32>,
    pub max_files: Option<u32>,
    pub stage_mode: Option<StageMode>,
    pub confirm: Option<bool>,
    pub temperature: Option<f32>,
    pub ignore: Option<Vec<String>>,
}

impl Config {
    #[must_use]
    pub fn defaults() -> Self {
        Self {
            provider: Some(ProviderKind::Ollama),
            model: Some("qwen2.5-coder:1.5b".to_string()),
            openai_mode: Some(OpenAiMode::Auto),
            openai_base_url: Some("https://api.openai.com/v1".to_string()),
            openai_api_key: None,
            ollama_endpoint: Some("http://localhost:11434/api/chat".to_string()),
            conventional: Some(true),
            one_line: Some(true),
            emoji: Some(false),
            lang: None,
            push: Some(true),
            timeout_secs: Some(20),
            max_input_tokens: Some(6000),
            max_output_tokens: Some(2048),
            max_file_bytes: Some(200_000),
            max_file_lines: Some(2_000),
            summary_concurrency: Some(4),
            max_files: Some(40),
            stage_mode: Some(StageMode::Auto),
            confirm: Some(true),
            temperature: Some(0.2),
            ignore: Some(Vec::new()),
        }
    }

    #[must_use]
    pub fn merge(self, other: Config) -> Self {
        Self {
            provider: other.provider.or(self.provider),
            model: other.model.or(self.model),
            openai_mode: other.openai_mode.or(self.openai_mode),
            openai_base_url: other.openai_base_url.or(self.openai_base_url),
            openai_api_key: other.openai_api_key.or(self.openai_api_key),
            ollama_endpoint: other.ollama_endpoint.or(self.ollama_endpoint),
            conventional: other.conventional.or(self.conventional),
            one_line: other.one_line.or(self.one_line),
            emoji: other.emoji.or(self.emoji),
            lang: other.lang.or(self.lang),
            push: other.push.or(self.push),
            timeout_secs: other.timeout_secs.or(self.timeout_secs),
            max_input_tokens: other.max_input_tokens.or(self.max_input_tokens),
            max_output_tokens: other.max_output_tokens.or(self.max_output_tokens),
            max_file_bytes: other.max_file_bytes.or(self.max_file_bytes),
            max_file_lines: other.max_file_lines.or(self.max_file_lines),
            summary_concurrency: other.summary_concurrency.or(self.summary_concurrency),
            max_files: other.max_files.or(self.max_files),
            stage_mode: other.stage_mode.or(self.stage_mode),
            confirm: other.confirm.or(self.confirm),
            temperature: other.temperature.or(self.temperature),
            ignore: other.ignore.or(self.ignore),
        }
    }

    /// Resolve the merged config into concrete defaults.
    ///
    /// # Errors
    /// Returns an error when config values are inconsistent.
    pub fn resolve(self) -> CoreResult<EffectiveConfig> {
        let provider = self.provider.unwrap_or(ProviderKind::Ollama);
        let model = self
            .model
            .unwrap_or_else(|| "qwen2.5-coder:1.5b".to_string());
        let mut openai_mode = self.openai_mode.unwrap_or(OpenAiMode::Auto);
        if provider == ProviderKind::OpenAi && model.trim().to_lowercase().starts_with("gpt-5") {
            openai_mode = OpenAiMode::Responses;
        }

        Ok(EffectiveConfig {
            provider,
            model,
            openai_mode,
            openai_base_url: self
                .openai_base_url
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            openai_api_key: self.openai_api_key,
            ollama_endpoint: self
                .ollama_endpoint
                .unwrap_or_else(|| "http://localhost:11434/api/chat".to_string()),
            conventional: self.conventional.unwrap_or(true),
            one_line: self.one_line.unwrap_or(true),
            emoji: self.emoji.unwrap_or(false),
            lang: self.lang,
            push: self.push.unwrap_or(true),
            timeout_secs: self.timeout_secs.unwrap_or(20),
            max_input_tokens: self.max_input_tokens.unwrap_or(6000),
            max_output_tokens: self.max_output_tokens.unwrap_or(2048),
            max_file_bytes: self.max_file_bytes.unwrap_or(200_000),
            max_file_lines: self.max_file_lines.unwrap_or(2_000),
            summary_concurrency: self.summary_concurrency.unwrap_or(4) as usize,
            max_files: self.max_files.unwrap_or(40) as usize,
            stage_mode: self.stage_mode.unwrap_or(StageMode::Auto),
            confirm: self.confirm.unwrap_or(true),
            temperature: self.temperature.unwrap_or(0.2),
            ignore: self.ignore.unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub provider: ProviderKind,
    pub model: String,
    pub openai_mode: OpenAiMode,
    pub openai_base_url: String,
    pub openai_api_key: Option<String>,
    pub ollama_endpoint: String,
    pub conventional: bool,
    pub one_line: bool,
    pub emoji: bool,
    pub lang: Option<String>,
    pub push: bool,
    pub timeout_secs: u64,
    pub max_input_tokens: u32,
    pub max_output_tokens: u32,
    pub max_file_bytes: u64,
    pub max_file_lines: u32,
    pub summary_concurrency: usize,
    pub max_files: usize,
    pub stage_mode: StageMode,
    pub confirm: bool,
    pub temperature: f32,
    pub ignore: Vec<String>,
}

impl EffectiveConfig {
    #[must_use]
    pub fn to_config(&self) -> Config {
        Config {
            provider: Some(self.provider),
            model: Some(self.model.clone()),
            openai_mode: Some(self.openai_mode),
            openai_base_url: Some(self.openai_base_url.clone()),
            openai_api_key: self.openai_api_key.clone(),
            ollama_endpoint: Some(self.ollama_endpoint.clone()),
            conventional: Some(self.conventional),
            one_line: Some(self.one_line),
            emoji: Some(self.emoji),
            lang: self.lang.clone(),
            push: Some(self.push),
            timeout_secs: Some(self.timeout_secs),
            max_input_tokens: Some(self.max_input_tokens),
            max_output_tokens: Some(self.max_output_tokens),
            max_file_bytes: Some(self.max_file_bytes),
            max_file_lines: Some(self.max_file_lines),
            summary_concurrency: Some(u32::try_from(self.summary_concurrency).unwrap_or(u32::MAX)),
            max_files: Some(u32::try_from(self.max_files).unwrap_or(u32::MAX)),
            stage_mode: Some(self.stage_mode),
            confirm: Some(self.confirm),
            temperature: Some(self.temperature),
            ignore: Some(self.ignore.clone()),
        }
    }
}
