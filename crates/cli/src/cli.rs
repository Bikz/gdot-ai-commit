use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::{ArgAction, Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use tracing_subscriber::EnvFilter;

use goodcommit_core::config::{
    config_dir, config_from_env, load_config, resolve_paths, Config, EffectiveConfig, ProviderKind,
    StageMode,
};
use goodcommit_core::git::{GitBackend, SystemGit};
use goodcommit_core::ignore::build_ignore_matcher;
use goodcommit_core::pipeline::{generate_commit_message, PipelineResult};
use goodcommit_core::providers::build_provider;

use crate::hooks;
use crate::setup;
use crate::ui;
use crate::util::{is_interactive, join_message_args};

#[derive(Parser, Debug)]
#[command(
    name = "goodcommit",
    version,
    about = "Good Commit: fast AI commit messages"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(value_name = "message", trailing_var_arg = true)]
    message: Vec<String>,

    #[arg(long)]
    provider: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    openai_mode: Option<String>,
    #[arg(long)]
    openai_base_url: Option<String>,
    #[arg(long)]
    ollama_endpoint: Option<String>,
    #[arg(long)]
    timeout: Option<u64>,
    #[arg(long)]
    max_input_tokens: Option<u32>,
    #[arg(long)]
    max_output_tokens: Option<u32>,
    #[arg(long)]
    max_file_bytes: Option<u64>,
    #[arg(long)]
    max_file_lines: Option<u32>,
    #[arg(long)]
    summary_concurrency: Option<u32>,
    #[arg(long)]
    max_files: Option<u32>,
    #[arg(long)]
    lang: Option<String>,

    #[arg(short = 'l', long, action = ArgAction::SetTrue)]
    local: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    conventional: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    no_conventional: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    one_line: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    no_one_line: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    emoji: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    no_emoji: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    push: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    no_push: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    stage_all: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    no_stage: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    interactive: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    yes: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    dry_run: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    edit: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    no_verify: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    skip_verify: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Config,
    Doctor,
    #[command(alias = "init")]
    Setup,
    Split,
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand, Debug)]
enum HookAction {
    Install,
    Uninstall,
    #[command(hide = true)]
    Run {
        path: PathBuf,
        source: Option<String>,
        sha: Option<String>,
    },
}

pub async fn run() -> Result<()> {
    let mut cli = Cli::parse();
    init_tracing(cli.verbose);

    let command = cli.command.take();

    match command {
        Some(Commands::Setup) => {
            setup::run_setup()?;
            ui::success("setup complete");
            return Ok(());
        }
        Some(Commands::Config) => {
            run_config(&cli)?;
            return Ok(());
        }
        Some(Commands::Doctor) => {
            run_doctor(&cli)?;
            return Ok(());
        }
        Some(Commands::Split) => {
            run_split(cli).await?;
            return Ok(());
        }
        Some(Commands::Hook { action }) => match action {
            HookAction::Install => {
                let git = SystemGit::new();
                git.ensure_git_repo()?;
                hooks::install_hook(&git)?;
                ui::success("hook installed");
                return Ok(());
            }
            HookAction::Uninstall => {
                let git = SystemGit::new();
                git.ensure_git_repo()?;
                hooks::uninstall_hook(&git)?;
                ui::success("hook removed");
                return Ok(());
            }
            HookAction::Run { path, source, .. } => {
                run_hook(path, source, cli).await?;
                return Ok(());
            }
        },
        None => {}
    }

    run_commit(cli).await
}

fn init_tracing(verbose: bool) {
    let default_filter = if verbose {
        "goodcommit=debug,goodcommit_core=debug"
    } else {
        "goodcommit=info,goodcommit_core=info"
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn build_cli_overrides(cli: &Cli) -> Result<Config> {
    let mut config = Config::default();

    if let Some(provider) = &cli.provider {
        config.provider = Some(provider.parse().map_err(|err: String| anyhow!(err))?);
    }

    if let Some(model) = &cli.model {
        config.model = Some(model.clone());
    }

    if let Some(mode) = &cli.openai_mode {
        config.openai_mode = Some(mode.parse().map_err(|err: String| anyhow!(err))?);
    }

    if let Some(base_url) = &cli.openai_base_url {
        config.openai_base_url = Some(base_url.clone());
    }

    if let Some(endpoint) = &cli.ollama_endpoint {
        config.ollama_endpoint = Some(endpoint.clone());
    }

    if let Some(timeout) = cli.timeout {
        config.timeout_secs = Some(timeout);
    }

    if let Some(max_input) = cli.max_input_tokens {
        config.max_input_tokens = Some(max_input);
    }

    if let Some(max_output) = cli.max_output_tokens {
        config.max_output_tokens = Some(max_output);
    }

    if let Some(max_file_bytes) = cli.max_file_bytes {
        config.max_file_bytes = Some(max_file_bytes);
    }

    if let Some(max_file_lines) = cli.max_file_lines {
        config.max_file_lines = Some(max_file_lines);
    }

    if let Some(summary_concurrency) = cli.summary_concurrency {
        config.summary_concurrency = Some(summary_concurrency);
    }

    if let Some(max_files) = cli.max_files {
        config.max_files = Some(max_files);
    }

    if let Some(lang) = &cli.lang {
        config.lang = Some(lang.clone());
    }

    if cli.conventional {
        config.conventional = Some(true);
    }
    if cli.no_conventional {
        config.conventional = Some(false);
    }

    if cli.one_line {
        config.one_line = Some(true);
    }
    if cli.no_one_line {
        config.one_line = Some(false);
    }

    if cli.emoji {
        config.emoji = Some(true);
    }
    if cli.no_emoji {
        config.emoji = Some(false);
    }

    if cli.local {
        config.push = Some(false);
    }

    if cli.push {
        config.push = Some(true);
    }
    if cli.no_push {
        config.push = Some(false);
    }

    if cli.yes {
        config.confirm = Some(false);
    }

    if cli.stage_all {
        config.stage_mode = Some(StageMode::All);
    }
    if cli.no_stage {
        config.stage_mode = Some(StageMode::None);
    }
    if cli.interactive {
        config.stage_mode = Some(StageMode::Interactive);
    }

    Ok(config)
}

fn stage_mode_conflicts(cli: &Cli) -> Result<()> {
    let mut count = 0;
    if cli.stage_all {
        count += 1;
    }
    if cli.no_stage {
        count += 1;
    }
    if cli.interactive {
        count += 1;
    }

    if count > 1 {
        Err(anyhow!("stage flags are mutually exclusive"))
    } else {
        Ok(())
    }
}

fn has_stage_flag(cli: &Cli) -> bool {
    cli.stage_all || cli.no_stage || cli.interactive
}

fn stage_mode_for_invocation(invocation: &str) -> Option<StageMode> {
    let name = Path::new(invocation)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(invocation);
    match name {
        "g." => Some(StageMode::All),
        _ => None,
    }
}

fn invocation_stage_mode() -> Option<StageMode> {
    std::env::args()
        .next()
        .and_then(|arg0| stage_mode_for_invocation(&arg0))
}

fn config_for_repo(
    cli: &Cli,
    repo_root: Option<&std::path::Path>,
) -> Result<(EffectiveConfig, goodcommit_core::config::ConfigPaths)> {
    stage_mode_conflicts(cli)?;

    let paths = resolve_paths(repo_root)?;
    let file_config = load_config(&paths)?;
    let env_config = config_from_env();
    let mut cli_config = build_cli_overrides(cli)?;
    if !has_stage_flag(cli) {
        if let Some(stage_mode) = invocation_stage_mode() {
            cli_config.stage_mode = Some(stage_mode);
        }
    }

    let config = Config::defaults()
        .merge(env_config)
        .merge(file_config)
        .merge(cli_config)
        .resolve()?;

    Ok((config, paths))
}

async fn run_commit(cli: Cli) -> Result<()> {
    if maybe_setup_from_message(&cli)? {
        return Ok(());
    }

    let git = SystemGit::new();
    git.ensure_git_repo()?;
    let repo_root = git.repo_root()?;
    maybe_prompt_setup(&cli, Some(&repo_root))?;
    let (config, paths) = config_for_repo(&cli, Some(&repo_root))?;

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

async fn run_split(cli: Cli) -> Result<()> {
    if !is_interactive() {
        return Err(anyhow!("split requires an interactive terminal"));
    }

    let git = SystemGit::new();
    git.ensure_git_repo()?;
    let repo_root = git.repo_root()?;
    maybe_prompt_setup(&cli, Some(&repo_root))?;
    let (mut config, paths) = config_for_repo(&cli, Some(&repo_root))?;
    config.stage_mode = StageMode::None;

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

async fn run_hook(path: PathBuf, source: Option<String>, cli: Cli) -> Result<()> {
    let git = SystemGit::new();
    git.ensure_git_repo()?;
    let repo_root = git.repo_root()?;
    let (mut config, paths) = config_for_repo(&cli, Some(&repo_root))?;

    config.confirm = false;
    config.push = false;
    config.stage_mode = StageMode::None;

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
    let provider = match build_provider(&config) {
        Ok(provider) => Some(provider),
        Err(_) => None,
    };

    let pipeline_result =
        generate_commit_message(&git, provider.as_deref(), &config, &ignore_matcher).await?;

    let outcome = match pipeline_result {
        PipelineResult::NoChanges => return Ok(()),
        PipelineResult::Message(outcome) => outcome,
    };

    hooks::write_hook_message(&path, &outcome.message)?;
    Ok(())
}

fn maybe_prompt_setup(cli: &Cli, repo_root: Option<&std::path::Path>) -> Result<()> {
    if !is_interactive() || cli.yes {
        return Ok(());
    }

    let paths = resolve_paths(repo_root)?;
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

fn run_config(cli: &Cli) -> Result<()> {
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

fn run_doctor(cli: &Cli) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_mode_for_invocation_matches_aliases() {
        assert_eq!(stage_mode_for_invocation("g."), Some(StageMode::All));
        assert_eq!(
            stage_mode_for_invocation("/opt/homebrew/bin/g."),
            Some(StageMode::All)
        );
        assert_eq!(stage_mode_for_invocation("g"), None);
        assert_eq!(stage_mode_for_invocation("/opt/homebrew/bin/g"), None);
        assert_eq!(stage_mode_for_invocation("goodcommit"), None);
    }
}
