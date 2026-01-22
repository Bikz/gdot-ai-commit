use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use tracing::info_span;

use goodcommit_core::config::{config_dir, EffectiveConfig, ProviderKind, StageMode};
use goodcommit_core::git::{GitBackend, SystemGit};
use goodcommit_core::ignore::build_ignore_matcher;
use goodcommit_core::pipeline::{generate_commit_message, PipelineResult};
use goodcommit_core::providers::build_provider;

use crate::hooks;
use crate::setup;
use crate::ui;
use crate::util::{is_interactive, join_message_args};

use super::args::Cli;
use super::config::config_for_repo;

pub(crate) async fn run_commit(cli: Cli) -> Result<()> {
    if maybe_setup_from_message(&cli)? {
        return Ok(());
    }

    let git = SystemGit::new();
    git.ensure_git_repo()?;
    let repo_root = git.repo_root()?;
    maybe_prompt_setup(&cli, Some(&repo_root))?;
    let (config, paths) = config_for_repo(&cli, Some(&repo_root))?;

    let span = info_span!(
        "commit_run",
        run_id = %generate_run_id(),
        provider = %config.provider.as_str(),
        model = %config.model,
        stage_mode = ?config.stage_mode,
    );
    let _enter = span.enter();

    let ignore_matcher = build_ignore_matcher(&config.ignore, &paths)?;

    match config.stage_mode {
        StageMode::All => git.stage_all()?,
        StageMode::Interactive => git.stage_interactive()?,
        StageMode::None => {}
        StageMode::Auto => {
            let staged_files = git.staged_files()?;
            if staged_files.is_empty() {
                git.stage_all()?;
            }
        }
    }

    if let Some(message) = join_message_args(&cli.message) {
        return commit_with_message(&git, &config, &cli, &message);
    }

    let provider = match build_provider(&config) {
        Ok(provider) => Some(provider),
        Err(err) => {
            ui::warn(&format!("provider setup failed, using fallback: {err}"));
            print_provider_help(&config);
            None
        }
    };

    let pipeline_result =
        generate_commit_message(&git, provider.as_deref(), &config, &ignore_matcher).await?;

    let outcome = match pipeline_result {
        PipelineResult::NoChanges => {
            if git.has_unstaged_changes()? {
                ui::warn("no staged changes; stage files or use --stage-all");
            } else {
                ui::info("working tree clean");
            }
            return Ok(());
        }
        PipelineResult::Message(outcome) => outcome,
    };

    for warning in &outcome.warnings {
        ui::warn(warning);
    }
    if has_provider_warning(&outcome.warnings) {
        print_provider_help(&config);
    }

    commit_with_message(&git, &config, &cli, &outcome.message)
}

pub(crate) async fn run_split(cli: Cli) -> Result<()> {
    if !is_interactive() {
        return Err(anyhow!("split requires an interactive terminal"));
    }

    let git = SystemGit::new();
    git.ensure_git_repo()?;
    let repo_root = git.repo_root()?;
    maybe_prompt_setup(&cli, Some(&repo_root))?;
    let (mut config, paths) = config_for_repo(&cli, Some(&repo_root))?;
    config.stage_mode = StageMode::None;

    let span = info_span!(
        "split_run",
        run_id = %generate_run_id(),
        provider = %config.provider.as_str(),
        model = %config.model,
        stage_mode = ?config.stage_mode,
    );
    let _enter = span.enter();

    let staged = git.staged_files()?;
    if !staged.is_empty() {
        ui::warn("staged changes detected");
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("unstage all and continue with split?")
            .default(false)
            .interact()?;
        if !confirm {
            ui::info("split canceled");
            return Ok(());
        }
        git.unstage_all()?;
    }

    let ignore_matcher = build_ignore_matcher(&config.ignore, &paths)?;

    let provider = match build_provider(&config) {
        Ok(provider) => Some(provider),
        Err(err) => {
            ui::warn(&format!("provider setup failed, using fallback: {err}"));
            print_provider_help(&config);
            None
        }
    };

    loop {
        let mut remaining = git.working_tree_files()?;
        if remaining.is_empty() {
            ui::info("working tree clean");
            return Ok(());
        }
        remaining.sort();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select files for next commit (space to select)")
            .items(&remaining)
            .interact()?;

        if selections.is_empty() {
            let done = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("no files selected; finish split?")
                .default(true)
                .interact()?;
            if done {
                ui::info("split complete");
                return Ok(());
            }
            continue;
        }

        let chosen: Vec<String> = selections
            .iter()
            .map(|index| remaining[*index].clone())
            .collect();

        git.stage_paths(&chosen)?;

        let pipeline_result =
            generate_commit_message(&git, provider.as_deref(), &config, &ignore_matcher).await?;

        let outcome = match pipeline_result {
            PipelineResult::NoChanges => {
                ui::warn("no staged diff for selection");
                git.unstage_all()?;
                continue;
            }
            PipelineResult::Message(outcome) => outcome,
        };

        for warning in &outcome.warnings {
            ui::warn(warning);
        }
        if has_provider_warning(&outcome.warnings) {
            print_provider_help(&config);
        }

        commit_with_message(&git, &config, &cli, &outcome.message)?;
        git.unstage_all()?;

        if cli.dry_run {
            return Ok(());
        }
    }
}

pub(crate) async fn run_hook(
    path: std::path::PathBuf,
    source: Option<String>,
    cli: Cli,
) -> Result<()> {
    let git = SystemGit::new();
    git.ensure_git_repo()?;
    let repo_root = git.repo_root()?;
    let (mut config, paths) = config_for_repo(&cli, Some(&repo_root))?;

    config.confirm = false;
    config.push = false;
    config.stage_mode = StageMode::None;

    let span = info_span!(
        "hook_run",
        run_id = %generate_run_id(),
        provider = %config.provider.as_str(),
        model = %config.model,
        stage_mode = ?config.stage_mode,
    );
    let _enter = span.enter();

    if let Some(source) = source.as_deref() {
        if !source.trim().is_empty() {
            return Ok(());
        }
    }

    if let Ok(existing) = std::fs::read_to_string(&path) {
        let has_message = existing
            .lines()
            .map(str::trim)
            .any(|line| !line.is_empty() && !line.starts_with('#'));
        if has_message {
            return Ok(());
        }
    }

    let ignore_matcher = build_ignore_matcher(&config.ignore, &paths)?;
    let provider = build_provider(&config).ok();

    let pipeline_result =
        generate_commit_message(&git, provider.as_deref(), &config, &ignore_matcher).await?;

    let outcome = match pipeline_result {
        PipelineResult::NoChanges => return Ok(()),
        PipelineResult::Message(outcome) => outcome,
    };

    hooks::write_hook_message(&path, &outcome.message)?;
    Ok(())
}

fn maybe_setup_from_message(cli: &Cli) -> Result<bool> {
    if cli.message.len() == 2
        && cli.message[0].eq_ignore_ascii_case("set")
        && cli.message[1].eq_ignore_ascii_case("up")
        && is_interactive()
    {
        ui::info("did you mean `goodcommit setup`?");
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("run setup now?")
            .default(true)
            .interact()?;
        if confirm {
            setup::run_setup()?;
            ui::success("setup complete");
            return Ok(true);
        }
    }

    Ok(false)
}

fn commit_with_message(
    git: &impl GitBackend,
    config: &EffectiveConfig,
    cli: &Cli,
    message: &str,
) -> Result<()> {
    ui::info("commit message preview:");
    ui::preview_message(message);

    if cli.dry_run {
        ui::info("dry run enabled; skipping commit");
        return Ok(());
    }

    if config.confirm && is_interactive() {
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("commit with this message?")
            .default(true)
            .interact()?;
        if !confirm {
            ui::info("commit canceled");
            return Ok(());
        }
    }

    let no_verify = cli.no_verify || cli.skip_verify;
    let output = git.commit(message, cli.edit, no_verify)?;
    if !output.is_empty() {
        ui::info(&output);
    }

    if config.push && !cli.no_push {
        match git.push() {
            Ok(push_output) => {
                if !push_output.is_empty() {
                    ui::info(&push_output);
                }
            }
            Err(err) => {
                ui::warn(&format!("push failed: {err}"));
            }
        }
    }

    Ok(())
}

fn has_provider_warning(warnings: &[String]) -> bool {
    warnings
        .iter()
        .any(|warning| warning.contains("ai generation failed") || warning.contains("provider"))
}

fn print_provider_help(config: &EffectiveConfig) {
    match config.provider {
        ProviderKind::OpenAi => {
            ui::info("fix: set OPENAI_API_KEY or GOODCOMMIT_OPENAI_API_KEY");
            ui::info("or run `goodcommit setup` to store a key or switch providers");
            if let Ok(dir) = config_dir() {
                let path = dir.join("config.toml");
                ui::info(&format!("config file: {}", path.display()));
            }
        }
        ProviderKind::Ollama => {
            ui::info("fix: install and run ollama (https://ollama.com)");
            ui::info("start it with: ollama serve");
            ui::info("or run `goodcommit setup` to switch providers");
        }
    }
}

fn maybe_prompt_setup(cli: &Cli, repo_root: Option<&std::path::Path>) -> Result<()> {
    if !is_interactive() || cli.yes {
        return Ok(());
    }

    let paths = goodcommit_core::config::resolve_paths(repo_root)?;
    let has_config = paths.global_config.is_some() || paths.repo_config.is_some();
    if has_config {
        return Ok(());
    }

    ui::info("no config found; run guided setup to choose provider and push defaults");
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("run setup now?")
        .default(true)
        .interact()?;

    if confirm {
        setup::run_setup()?;
        ui::success("setup complete");
    }

    Ok(())
}

fn generate_run_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}-{}", now.as_millis(), std::process::id())
}
