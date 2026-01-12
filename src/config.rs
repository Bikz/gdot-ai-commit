use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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
            max_output_tokens: Some(200),
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
            stage_mode: other.stage_mode.or(self.stage_mode),
            confirm: other.confirm.or(self.confirm),
            temperature: other.temperature.or(self.temperature),
            ignore: other.ignore.or(self.ignore),
        }
    }

    pub fn resolve(self) -> Result<EffectiveConfig> {
        Ok(EffectiveConfig {
            provider: self.provider.unwrap_or(ProviderKind::Ollama),
            model: self
                .model
                .unwrap_or_else(|| "qwen2.5-coder:1.5b".to_string()),
            openai_mode: self.openai_mode.unwrap_or(OpenAiMode::Auto),
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
            max_output_tokens: self.max_output_tokens.unwrap_or(200),
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

pub fn config_dir() -> Result<PathBuf> {
    if let Ok(home) = env::var("HOME") {
        return Ok(PathBuf::from(home).join(".config").join("goodcommit"));
    }

    if let Ok(userprofile) = env::var("USERPROFILE") {
        return Ok(PathBuf::from(userprofile)
            .join(".config")
            .join("goodcommit"));
    }

    Err(anyhow::anyhow!("unable to resolve config directory"))
}

pub fn resolve_paths(repo_root: Option<&Path>) -> Result<ConfigPaths> {
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

pub fn load_config(paths: &ConfigPaths) -> Result<Config> {
    let mut config = Config::default();

    if let Some(path) = &paths.global_config {
        config = config.merge(read_config_file(path)?);
    }

    if let Some(path) = &paths.repo_config {
        config = config.merge(read_config_file(path)?);
    }

    Ok(config)
}

pub fn read_config_file(path: &Path) -> Result<Config> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed reading config {path:?}"))?;

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => toml::from_str(&content).context("failed parsing toml config"),
        Some("yaml") | Some("yml") => {
            serde_yaml::from_str(&content).context("failed parsing yaml config")
        }
        _ => toml::from_str(&content).context("failed parsing config"),
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
