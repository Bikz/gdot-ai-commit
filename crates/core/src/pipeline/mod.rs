use std::time::{Duration, Instant};

use tracing::{debug, instrument, warn};

use crate::config::EffectiveConfig;
use crate::error::CoreResult;
use crate::git::GitBackend;
use crate::ignore::IgnoreMatcher;
use crate::providers::Provider;

mod context;
mod generation;
mod sanitize;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum PipelineResult {
    NoChanges,
    Message(PipelineOutcome),
}

#[derive(Debug)]
pub struct PipelineOutcome {
    pub message: String,
    pub used_fallback: bool,
    pub warnings: Vec<String>,
}

#[instrument(level = "info", skip(git, provider, config, ignore))]
/// Generate a commit message using staged changes and the configured provider.
///
/// # Errors
/// Returns an error if git access fails, the provider fails, or timeouts occur.
pub async fn generate_commit_message(
    git: &impl GitBackend,
    provider: Option<&dyn Provider>,
    config: &EffectiveConfig,
    ignore: &IgnoreMatcher,
) -> CoreResult<PipelineResult> {
    let start = Instant::now();
    let context = context::collect_diff_context(git, config, ignore)?;
    if context.all_paths.is_empty() {
        return Ok(PipelineResult::NoChanges);
    }

    let fallback = fallback_message(&context.all_paths, config);
    if context.ai_files.is_empty() {
        let mut warnings = context.warnings;
        warnings.push("no usable diff for AI; using fallback".to_string());
        return Ok(PipelineResult::Message(PipelineOutcome {
            message: fallback,
            used_fallback: true,
            warnings,
        }));
    }

    let mut warnings = context.warnings;
    let deadline = Instant::now() + Duration::from_secs(config.timeout_secs);

    let message = if let Some(provider) = provider {
        match generation::generate_with_provider(provider, config, &context.ai_files, deadline)
            .await
        {
            Ok(message) => message,
            Err(err) => {
                warn!("ai generation failed: {err}");
                warnings.push(format!("ai generation failed, using fallback: {err}"));
                fallback.clone()
            }
        }
    } else {
        warnings.push("provider unavailable, using fallback".to_string());
        fallback.clone()
    };

    let cleaned = sanitize::sanitize_message(&message, config, &fallback);
    let used_fallback = cleaned == fallback;

    debug!(
        elapsed_ms = start.elapsed().as_millis(),
        "pipeline complete"
    );

    Ok(PipelineResult::Message(PipelineOutcome {
        message: cleaned,
        used_fallback,
        warnings,
    }))
}

fn fallback_message(paths: &[String], config: &EffectiveConfig) -> String {
    let mut subject = if paths.is_empty() {
        "update files".to_string()
    } else {
        let preview = paths.iter().take(3).cloned().collect::<Vec<_>>();
        format!("update {}", preview.join(", "))
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
