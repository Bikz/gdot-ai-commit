use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};

use crate::config::{config_dir, openai_api_key, Config, ProviderKind};
use crate::ignore::default_patterns;
use crate::util::is_interactive;

pub fn run_setup() -> Result<()> {
    if !is_interactive() {
        return Err(anyhow!("setup requires an interactive terminal"));
    }

    let theme = ColorfulTheme::default();
    let config_dir = config_dir()?;
    fs::create_dir_all(&config_dir).context("failed to create config directory")?;

    let config_path = config_dir.join("config.toml");
    if config_path.exists() {
        let overwrite = Confirm::with_theme(&theme)
            .with_prompt("config.toml already exists. Overwrite?")
            .default(false)
            .interact()?;
        if !overwrite {
            return Ok(());
        }
    }

    let provider = Select::with_theme(&theme)
        .with_prompt("Choose your default provider")
        .items(&["ollama (local)", "openai"])
        .default(0)
        .interact()?;

    let (provider_kind, default_model) = if provider == 1 {
        (ProviderKind::OpenAi, "gpt-5-nano-2025-08-07")
    } else {
        (ProviderKind::Ollama, "qwen2.5-coder:1.5b")
    };

    if provider_kind == ProviderKind::OpenAi && openai_api_key().is_none() {
        eprintln!("warning: OPENAI_API_KEY not set; set it before using OpenAI models");
    }

    let model: String = Input::with_theme(&theme)
        .with_prompt("Default model")
        .default(default_model.to_string())
        .interact_text()?;

    let push = Confirm::with_theme(&theme)
        .with_prompt("Push by default after commit?")
        .default(true)
        .interact()?;

    let mut config = Config::default();
    config.provider = Some(provider_kind);
    config.model = Some(model);
    config.push = Some(push);
    config.conventional = Some(true);
    config.one_line = Some(true);
    config.timeout_secs = Some(20);
    config.max_input_tokens = Some(6000);
    config.max_output_tokens = Some(200);
    config.stage_mode = Some(crate::config::StageMode::Auto);

    let toml = toml::to_string_pretty(&config).context("failed to serialize config")?;
    fs::write(&config_path, toml).context("failed to write config")?;

    ensure_ignore_file(&config_dir.join("ignore"))?;

    Ok(())
}

fn ensure_ignore_file(path: &PathBuf) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    let patterns = default_patterns();
    let content = patterns.join("\n") + "\n";
    fs::write(path, content).context("failed to write ignore file")
}
