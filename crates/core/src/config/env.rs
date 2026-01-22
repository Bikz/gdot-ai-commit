use std::env;

use super::types::StageMode;
use super::values::Config;

#[must_use]
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
        if let Ok(stage) = value.parse::<StageMode>() {
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

/// Parse a boolean flag from a string.
///
/// # Errors
/// Returns an error when the value is not a recognized boolean.
pub fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid bool: {value}")),
    }
}

#[must_use]
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
