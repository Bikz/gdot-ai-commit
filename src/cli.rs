use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{ArgAction, Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm};
use regex::Regex;

use crate::config::{
    config_from_env, load_config, resolve_paths, Config, EffectiveConfig, ProviderKind, StageMode,
};
use crate::diff::{
    diff_files_to_string, estimate_tokens, filter_diff_files, parse_diff, truncate_to_tokens,
    DiffFile,
};
use crate::git;
use crate::hooks;
use crate::ignore::build_ignore_matcher;
use crate::prompt::{
    commit_system_prompt, commit_user_prompt, summary_system_prompt, summary_user_prompt,
};
use crate::providers::{build_provider, Provider, ProviderRequest};
use crate::setup;
use crate::ui;
use crate::util::{is_interactive, join_message_args, trim_quotes};

#[derive(Parser, Debug)]
#[command(
    name = "git-ai-commit",
    version,
    about = "One-command AI commit messages"
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
    lang: Option<String>,

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
}

#[derive(Subcommand, Debug)]
enum Commands {
    Config,
    Doctor,
    Setup,
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
        Some(Commands::Hook { action }) => match action {
            HookAction::Install => {
                git::ensure_git_repo()?;
                hooks::install_hook()?;
                ui::success("hook installed");
                return Ok(());
            }
            HookAction::Uninstall => {
                git::ensure_git_repo()?;
                hooks::uninstall_hook()?;
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

fn config_for_repo(
    cli: &Cli,
    repo_root: Option<&std::path::Path>,
) -> Result<(EffectiveConfig, crate::config::ConfigPaths)> {
    stage_mode_conflicts(cli)?;

    let paths = resolve_paths(repo_root)?;
    let file_config = load_config(&paths)?;
    let env_config = config_from_env();
    let cli_config = build_cli_overrides(cli)?;

    let config = Config::defaults()
        .merge(env_config)
        .merge(file_config)
        .merge(cli_config)
        .resolve()?;

    Ok((config, paths))
}

async fn run_commit(cli: Cli) -> Result<()> {
    git::ensure_git_repo()?;
    let repo_root = git::repo_root()?;
    maybe_prompt_setup(&cli, Some(&repo_root))?;
    let (config, paths) = config_for_repo(&cli, Some(&repo_root))?;

    let ignore_matcher = build_ignore_matcher(&config.ignore, &paths)?;

    match config.stage_mode {
        StageMode::All => git::stage_all()?,
        StageMode::Interactive => git::stage_interactive()?,
        StageMode::None => {}
        StageMode::Auto => {
            let staged_files = git::staged_files()?;
            if staged_files.is_empty() {
                git::stage_all()?;
            }
        }
    }

    let staged_files = git::staged_files()?;
    if staged_files.is_empty() {
        if git::has_unstaged_changes()? {
            ui::warn("no staged changes; stage files or use --stage-all");
        } else {
            ui::info("working tree clean");
        }
        return Ok(());
    }

    let raw_diff = git::staged_diff()?;
    if raw_diff.trim().is_empty() {
        ui::info("no diff content for staged files");
        return Ok(());
    }

    let diff_files = filter_diff_files(parse_diff(&raw_diff), &ignore_matcher);
    let diff_for_prompt = diff_files_to_string(&diff_files);

    let message = if let Some(message) = join_message_args(&cli.message) {
        message
    } else if diff_for_prompt.trim().is_empty() {
        fallback_message(&diff_files, &config)
    } else {
        match build_provider(&config) {
            Ok(provider) => {
                match generate_commit_message(&*provider, &config, &diff_files, &diff_for_prompt)
                    .await
                {
                    Ok(message) => message,
                    Err(err) => {
                        ui::warn(&format!("ai generation failed, using fallback: {err}"));
                        fallback_message(&diff_files, &config)
                    }
                }
            }
            Err(err) => {
                ui::warn(&format!("provider setup failed, using fallback: {err}"));
                fallback_message(&diff_files, &config)
            }
        }
    };

    let fallback = fallback_message(&diff_files, &config);
    let cleaned = sanitize_message(&message, &config, &fallback);

    ui::info("commit message preview:");
    ui::preview_message(&cleaned);

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
    let output = git::commit(&cleaned, cli.edit, no_verify).context("git commit failed")?;
    if !output.is_empty() {
        ui::info(&output);
    }

    if config.push && !cli.no_push {
        match git::push() {
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

async fn run_hook(path: PathBuf, source: Option<String>, cli: Cli) -> Result<()> {
    git::ensure_git_repo()?;
    let repo_root = git::repo_root()?;
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
    let raw_diff = git::staged_diff()?;
    if raw_diff.trim().is_empty() {
        return Ok(());
    }

    let diff_files = filter_diff_files(parse_diff(&raw_diff), &ignore_matcher);
    let diff_for_prompt = diff_files_to_string(&diff_files);
    if diff_for_prompt.trim().is_empty() {
        return Ok(());
    }

    let provider = match build_provider(&config) {
        Ok(provider) => provider,
        Err(_) => return Ok(()),
    };
    let message =
        match generate_commit_message(&*provider, &config, &diff_files, &diff_for_prompt).await {
            Ok(message) => message,
            Err(_) => return Ok(()),
        };
    let fallback = fallback_message(&diff_files, &config);
    let cleaned = sanitize_message(&message, &config, &fallback);

    hooks::write_hook_message(&path, &cleaned)?;
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

async fn generate_commit_message(
    provider: &dyn Provider,
    config: &EffectiveConfig,
    diff_files: &[DiffFile],
    diff_for_prompt: &str,
) -> Result<String> {
    let diff_tokens = estimate_tokens(diff_for_prompt);
    if diff_tokens <= config.max_input_tokens as usize {
        let system_prompt = commit_system_prompt(config);
        let user_prompt = commit_user_prompt(diff_for_prompt, config);
        let request = ProviderRequest {
            max_output_tokens: config.max_output_tokens,
            temperature: config.temperature,
        };
        return provider
            .complete(&system_prompt, &user_prompt, request)
            .await;
    }

    summarize_then_commit(provider, config, diff_files).await
}

async fn summarize_then_commit(
    provider: &dyn Provider,
    config: &EffectiveConfig,
    diff_files: &[DiffFile],
) -> Result<String> {
    let max_files = 40usize;
    let max_file_tokens = std::cmp::min(config.max_input_tokens as usize, 2000);
    let summary_tokens = std::cmp::min(config.max_output_tokens, 120);

    let mut summaries = Vec::new();
    for file in diff_files.iter().take(max_files) {
        let truncated = truncate_to_tokens(&file.content, max_file_tokens);
        if truncated.trim().is_empty() {
            continue;
        }
        let system_prompt = summary_system_prompt();
        let user_prompt = summary_user_prompt(&file.path, &truncated);
        let request = ProviderRequest {
            max_output_tokens: summary_tokens,
            temperature: config.temperature,
        };
        let summary = provider
            .complete(&system_prompt, &user_prompt, request)
            .await?;
        summaries.push(format!("{}: {}", file.path, summary.trim()));
    }

    if summaries.is_empty() {
        return Ok(fallback_message(diff_files, config));
    }

    let mut combined = summaries.join("\n");
    let combined_tokens = estimate_tokens(&combined);
    if combined_tokens > config.max_input_tokens as usize {
        combined = truncate_to_tokens(&combined, config.max_input_tokens as usize);
    }

    let system_prompt = commit_system_prompt(config);
    let user_prompt = commit_user_prompt(&combined, config);
    let request = ProviderRequest {
        max_output_tokens: config.max_output_tokens,
        temperature: config.temperature,
    };

    provider
        .complete(&system_prompt, &user_prompt, request)
        .await
}

fn sanitize_message(raw: &str, config: &EffectiveConfig, fallback: &str) -> String {
    let cleaned = trim_quotes(raw);
    let mut message = cleaned.trim().to_string();

    if config.one_line {
        message = message.lines().next().unwrap_or("").trim().to_string();
    }

    message = message.replace("```", "").replace('`', "");

    if config.conventional {
        let re = conventional_regex();
        let first_line = message.lines().next().unwrap_or("").trim();
        if !re.is_match(first_line) {
            if let Some(found) = cleaned.lines().find(|line| re.is_match(line.trim())) {
                message = found.trim().to_string();
            } else {
                message = fallback.to_string();
            }
        }
    }

    if message.is_empty() {
        fallback.to_string()
    } else {
        message
    }
}

fn conventional_regex() -> &'static Regex {
    use once_cell::sync::Lazy;
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(feat|fix|build|chore|ci|docs|style|refactor|perf|test)(\([\w./-]+\))?: .+")
            .expect("invalid regex")
    });
    &RE
}

fn fallback_message(diff_files: &[DiffFile], config: &EffectiveConfig) -> String {
    let paths: Vec<String> = diff_files
        .iter()
        .map(|file| file.path.clone())
        .take(3)
        .collect();

    let mut subject = if paths.is_empty() {
        "update files".to_string()
    } else {
        format!("update {}", paths.join(", "))
    };

    if subject.len() > 50 {
        subject.truncate(50);
    }

    if config.conventional {
        format!("chore: {subject}")
    } else {
        subject
    }
}

fn run_config(cli: &Cli) -> Result<()> {
    let repo_root = git::repo_root().ok();
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
    let repo_root = git::repo_root().ok();
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
                ui::warn("openai api key: missing (run setup or set OPENAI_API_KEY)");
            }
        }
        ProviderKind::Ollama => {
            ui::info(&format!("ollama endpoint: {}", config.ollama_endpoint));
        }
    }

    Ok(())
}
