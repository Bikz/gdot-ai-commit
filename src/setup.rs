use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};

use crate::config::{config_dir, legacy_config_dir, openai_api_key_env, Config, ProviderKind};
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
    let legacy_config_path = legacy_config_dir().map(|dir| dir.join("config.toml"));
    if !config_path.exists() {
        if let Some(legacy_path) = legacy_config_path.as_ref() {
            if legacy_path.exists() {
                let migrate = Confirm::with_theme(&theme)
                    .with_prompt("Legacy config found. Migrate to the new location?")
                    .default(true)
                    .interact()?;
                if migrate {
                    fs::copy(legacy_path, &config_path).context("failed to copy legacy config")?;
                    set_config_permissions(&config_path)?;
                    let legacy_ignore = legacy_config_dir().map(|dir| dir.join("ignore"));
                    if let Some(ignore_path) = legacy_ignore {
                        let new_ignore = config_dir.join("ignore");
                        if ignore_path.exists() && !new_ignore.exists() {
                            fs::copy(ignore_path, &new_ignore)
                                .context("failed to copy legacy ignore")?;
                        }
                    }
                    ensure_ignore_file(&config_dir.join("ignore"))?;
                    return Ok(());
                }
            }
        }
    }
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

    let mut openai_key = None;
    if provider_kind == ProviderKind::OpenAi {
        let env_key = openai_api_key_env();
        let had_env_key = env_key.is_some();

        if let Some(existing) = env_key {
            eprintln!("OpenAI API key detected in your environment.");
            let save = Confirm::with_theme(&theme)
                .with_prompt("Save it to config.toml? (stored in plaintext)")
                .default(false)
                .interact()?;
            if save {
                openai_key = Some(existing);
            }
        } else {
            eprintln!("OpenAI API key required. Get one at:");
            eprintln!("https://platform.openai.com/api-keys");
            let key = Password::with_theme(&theme)
                .with_prompt("Enter OpenAI API key (stored in config.toml)")
                .allow_empty_password(true)
                .interact()?;
            if !key.trim().is_empty() {
                openai_key = Some(key);
            }
        }

        if openai_key.is_none() && !had_env_key {
            eprintln!("No OpenAI key saved. Set OPENAI_API_KEY or GOODCOMMIT_OPENAI_API_KEY.");
        }
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
    config.openai_api_key = openai_key;
    config.push = Some(push);
    config.conventional = Some(true);
    config.one_line = Some(true);
    config.timeout_secs = Some(20);
    config.max_input_tokens = Some(6000);
    config.max_output_tokens = Some(200);
    config.stage_mode = Some(crate::config::StageMode::Auto);

    let toml = toml::to_string_pretty(&config).context("failed to serialize config")?;
    fs::write(&config_path, toml).context("failed to write config")?;
    set_config_permissions(&config_path)?;

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

#[cfg(unix)]
fn set_config_permissions(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms).context("failed to set config permissions")
}

#[cfg(not(unix))]
fn set_config_permissions(_path: &PathBuf) -> Result<()> {
    Ok(())
}
