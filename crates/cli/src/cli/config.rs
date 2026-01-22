use std::path::Path;

use anyhow::{anyhow, Result};

use goodcommit_core::config::{
    config_from_env, load_config, resolve_paths, Config, ConfigPaths, EffectiveConfig, StageMode,
};

use super::args::Cli;

pub(crate) fn build_cli_overrides(cli: &Cli) -> Result<Config> {
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

pub(crate) fn config_for_repo(
    cli: &Cli,
    repo_root: Option<&Path>,
) -> Result<(EffectiveConfig, ConfigPaths)> {
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
