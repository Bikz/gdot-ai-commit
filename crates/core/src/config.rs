use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    OpenAi,
    Ollama,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenAi => "openai",
            ProviderKind::Ollama => "ollama",
        }
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "openai" => Ok(ProviderKind::OpenAi),
            "ollama" => Ok(ProviderKind::Ollama),
            other => Err(format!("unknown provider: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OpenAiMode {
    Auto,
    Responses,
    Chat,
}

impl std::str::FromStr for OpenAiMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "auto" => Ok(OpenAiMode::Auto),
            "responses" => Ok(OpenAiMode::Responses),
            "chat" => Ok(OpenAiMode::Chat),
            other => Err(format!("unknown openai mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageMode {
    Auto,
    All,
    None,
    Interactive,
}

impl std::str::FromStr for StageMode {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "auto" => Ok(StageMode::Auto),
            "all" => Ok(StageMode::All),
            "none" => Ok(StageMode::None),
            "interactive" => Ok(StageMode::Interactive),
            other => Err(format!("unknown stage mode: {other}")),
        }
    }
}

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
            summary_concurrency: Some(self.summary_concurrency as u32),
            max_files: Some(self.max_files as u32),
            stage_mode: Some(self.stage_mode),
            confirm: Some(self.confirm),
            temperature: Some(self.temperature),
            ignore: Some(self.ignore.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigPaths {
    pub global_config: Option<PathBuf>,
    pub repo_config: Option<PathBuf>,
    pub global_ignore: PathBuf,
    pub repo_ignore: Option<PathBuf>,
}

pub fn config_dir() -> CoreResult<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config").join("goodcommit"));
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        return Ok(PathBuf::from(userprofile)
            .join(".config")
            .join("goodcommit"));
    }

    Err(CoreError::Config(
        "unable to resolve config directory".to_string(),
    ))
}

pub fn resolve_paths(repo_root: Option<&Path>) -> CoreResult<ConfigPaths> {
    let config_dir = config_dir()?;

    let global_config =
        find_config_file(&config_dir, &["config.toml", "config.yaml", "config.yml"]);

    let repo_config = repo_root.and_then(|root| {
        find_config_file(
            root,
            &[".goodcommit.toml", ".goodcommit.yaml", ".goodcommit.yml"],
        )
    });

    let global_ignore = config_dir.join("ignore");
    let repo_ignore = repo_root.and_then(|root| {
        let path = root.join(".goodcommit-ignore");
        if path.exists() {
            Some(path)
        } else {
            None
        }
    });

    Ok(ConfigPaths {
        global_config,
        repo_config,
        global_ignore,
        repo_ignore,
    })
}

pub fn load_config(paths: &ConfigPaths) -> CoreResult<Config> {
    let mut config = Config::default();

    if let Some(path) = &paths.global_config {
        config = config.merge(read_config_file(path)?);
    }

    if let Some(path) = &paths.repo_config {
        config = config.merge(read_config_file(path)?);
    }

    Ok(config)
}

pub fn read_config_file(path: &Path) -> CoreResult<Config> {
    let content = fs::read_to_string(path).map_err(|err| {
        CoreError::Config(format!("failed reading config {}: {err}", path.display()))
    })?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => toml::from_str(&content)
            .map_err(|err| CoreError::Config(format!("failed parsing toml config: {err}"))),
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
            .map_err(|err| CoreError::Config(format!("failed parsing yaml config: {err}"))),
        _ => toml::from_str(&content)
            .map_err(|err| CoreError::Config(format!("failed parsing config: {err}"))),
    }
}

pub fn config_from_env() -> Config {
    let mut config = Config::default();

    if let Ok(value) = env::var("GOODCOMMIT_PROVIDER") {
        if let Ok(provider) = value.parse() {
            config.provider = Some(provider);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_MODEL") {
        config.model = Some(value);
    }

    if let Ok(value) = env::var("GOODCOMMIT_OPENAI_MODE") {
        if let Ok(mode) = value.parse() {
            config.openai_mode = Some(mode);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_OPENAI_BASE_URL") {
        config.openai_base_url = Some(value);
    }

    if let Some(value) = openai_api_key_env() {
        config.openai_api_key = Some(value);
    }

    if let Ok(value) = env::var("GOODCOMMIT_OLLAMA_ENDPOINT") {
        config.ollama_endpoint = Some(value);
    }

    if let Ok(value) = env::var("GOODCOMMIT_CONVENTIONAL") {
        if let Ok(flag) = parse_bool(&value) {
            config.conventional = Some(flag);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_ONE_LINE") {
        if let Ok(flag) = parse_bool(&value) {
            config.one_line = Some(flag);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_EMOJI") {
        if let Ok(flag) = parse_bool(&value) {
            config.emoji = Some(flag);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_LANG") {
        config.lang = Some(value);
    }

    if let Ok(value) = env::var("GOODCOMMIT_PUSH") {
        if let Ok(flag) = parse_bool(&value) {
            config.push = Some(flag);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_TIMEOUT_SECS") {
        if let Ok(parsed) = value.parse::<u64>() {
            config.timeout_secs = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_MAX_INPUT_TOKENS") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.max_input_tokens = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_MAX_OUTPUT_TOKENS") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.max_output_tokens = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_MAX_FILE_BYTES") {
        if let Ok(parsed) = value.parse::<u64>() {
            config.max_file_bytes = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_MAX_FILE_LINES") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.max_file_lines = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_SUMMARY_CONCURRENCY") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.summary_concurrency = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_MAX_FILES") {
        if let Ok(parsed) = value.parse::<u32>() {
            config.max_files = Some(parsed);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_STAGE") {
        if let Ok(stage) = value.parse() {
            config.stage_mode = Some(stage);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_CONFIRM") {
        if let Ok(flag) = parse_bool(&value) {
            config.confirm = Some(flag);
        }
    }

    if let Ok(value) = env::var("GOODCOMMIT_TEMPERATURE") {
        if let Ok(parsed) = value.parse::<f32>() {
            config.temperature = Some(parsed);
        }
    }

    config
}

pub fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid bool: {value}")),
    }
}

fn find_config_file(base: &Path, candidates: &[&str]) -> Option<PathBuf> {
    for name in candidates {
        let path = base.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn openai_api_key_env() -> Option<String> {
    env_any(&["GOODCOMMIT_OPENAI_API_KEY", "OPENAI_API_KEY"])
}

fn env_any(keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Ok(value) = env::var(key) {
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_overrides_defaults() {
        let base = Config::defaults();
        let override_config = Config {
            push: Some(false),
            ..Config::default()
        };

        let merged = base.merge(override_config).resolve().expect("resolve");
        assert!(!merged.push);
    }

    #[test]
    fn resolve_forces_responses_for_gpt5_openai() {
        let config = Config {
            provider: Some(ProviderKind::OpenAi),
            model: Some("gpt-5-nano-2025-08-07".to_string()),
            openai_mode: Some(OpenAiMode::Chat),
            ..Config::default()
        };

        let resolved = config.resolve().expect("resolve");
        assert_eq!(resolved.openai_mode, OpenAiMode::Responses);
    }
}
