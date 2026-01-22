use anyhow::Result;

use goodcommit_core::config::ProviderKind;
use goodcommit_core::git::{GitBackend, SystemGit};

use crate::ui;

use super::args::Cli;
use super::config::config_for_repo;

pub(crate) fn run_config(cli: &Cli) -> Result<()> {
    let git = SystemGit::new();
    let repo_root = git.repo_root().ok();
    let (config, paths) = config_for_repo(cli, repo_root.as_deref())?;

    if let Some(global) = paths.global_config {
        ui::info(&format!("global config: {}", global.display()));
    } else {
        ui::info("global config: (none)");
    }

    if let Some(repo) = paths.repo_config {
        ui::info(&format!("repo config: {}", repo.display()));
    } else {
        ui::info("repo config: (none)");
    }

    ui::info(&format!("global ignore: {}", paths.global_ignore.display()));
    if let Some(repo_ignore) = paths.repo_ignore {
        ui::info(&format!("repo ignore: {}", repo_ignore.display()));
    } else {
        ui::info("repo ignore: (none)");
    }

    let mut printable = config.to_config();
    if printable.openai_api_key.is_some() {
        printable.openai_api_key = Some("[redacted]".to_string());
    }
    let toml = toml::to_string_pretty(&printable)?;
    ui::info("effective config:");
    println!("{toml}");

    Ok(())
}

pub(crate) fn run_doctor(cli: &Cli) -> Result<()> {
    let git = SystemGit::new();
    let repo_root = git.repo_root().ok();
    let (config, _paths) = config_for_repo(cli, repo_root.as_deref())?;

    let git_version = std::process::Command::new("git")
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_else(|| "git not found".to_string());

    ui::info(&format!("git: {}", git_version.trim()));
    ui::info(&format!("provider: {}", config.provider.as_str()));
    ui::info(&format!("model: {}", config.model));

    match config.provider {
        ProviderKind::OpenAi => {
            if config.openai_api_key.is_some() {
                ui::info("openai api key: detected");
            } else {
                ui::warn(
                    "openai api key: missing (run setup or set OPENAI_API_KEY or GOODCOMMIT_OPENAI_API_KEY)",
                );
            }
        }
        ProviderKind::Ollama => {
            ui::info(&format!("ollama endpoint: {}", config.ollama_endpoint));
        }
    }

    Ok(())
}
