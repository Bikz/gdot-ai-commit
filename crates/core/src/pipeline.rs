use std::time::{Duration, Instant};

use futures::{stream, StreamExt};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::warn;

use crate::config::EffectiveConfig;
use crate::diff::{
    diff_files_to_string, estimate_tokens, truncate_lines, truncate_to_tokens, DiffFile,
};
use crate::error::{CoreError, CoreResult};
use crate::git::GitBackend;
use crate::ignore::IgnoreMatcher;
use crate::prompt::{
    commit_system_prompt, commit_user_prompt, summary_system_prompt, summary_user_prompt,
};
use crate::providers::{Provider, ProviderRequest};

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

pub async fn generate_commit_message(
    git: &impl GitBackend,
    provider: Option<&dyn Provider>,
    config: &EffectiveConfig,
    ignore: &IgnoreMatcher,
) -> CoreResult<PipelineResult> {
    let context = collect_diff_context(git, config, ignore)?;
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
        match generate_with_provider(provider, config, &context.ai_files, deadline).await {
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

    let cleaned = sanitize_message(&message, config, &fallback);
    let used_fallback = cleaned == fallback;

    Ok(PipelineResult::Message(PipelineOutcome {
        message: cleaned,
        used_fallback,
        warnings,
    }))
}

struct DiffContext {
    all_paths: Vec<String>,
    ai_files: Vec<DiffFile>,
    warnings: Vec<String>,
}

fn collect_diff_context(
    git: &impl GitBackend,
    config: &EffectiveConfig,
    ignore: &IgnoreMatcher,
) -> CoreResult<DiffContext> {
    let stats = git.staged_numstat()?;
    if stats.is_empty() {
        return Ok(DiffContext {
            all_paths: Vec::new(),
            ai_files: Vec::new(),
            warnings: Vec::new(),
        });
    }

    let mut warnings = Vec::new();
    let all_paths = stats
        .iter()
        .map(|stat| stat.path.clone())
        .collect::<Vec<_>>();

    let mut ai_files = Vec::new();
    if stats.len() > config.max_files {
        warnings.push(format!(
            "only first {} files used for AI summary",
            config.max_files
        ));
    }

    for stat in stats.into_iter().take(config.max_files) {
        if stat.is_binary {
            continue;
        }
        if ignore.is_ignored(&stat.path) {
            continue;
        }

        let path = stat.path;
        let additions = stat.additions;
        let deletions = stat.deletions;
        let change_lines = additions.saturating_add(deletions);
        if change_lines > config.max_file_lines {
            warnings.push(format!(
                "diff omitted for {} ({} lines)",
                &path, change_lines
            ));
            let content = format!(
                "file {} changed: +{} -{} (diff omitted due to size)",
                &path, additions, deletions
            );
            let token_estimate = estimate_tokens(&content);
            ai_files.push(DiffFile {
                path,
                content,
                is_binary: false,
                truncated: true,
                additions,
                deletions,
                token_estimate,
            });
            continue;
        }

        let diff = git.staged_diff_for_path(&path, config.max_file_bytes)?;
        let (content, truncated_by_lines) = truncate_lines(&diff.content, config.max_file_lines);
        let truncated = diff.truncated || truncated_by_lines;
        if content.trim().is_empty() {
            continue;
        }

        if truncated {
            warnings.push(format!("diff truncated for {}", &path));
        }

        let token_estimate = estimate_tokens(&content);
        ai_files.push(DiffFile {
            path,
            content,
            is_binary: false,
            truncated,
            additions,
            deletions,
            token_estimate,
        });
    }

    Ok(DiffContext {
        all_paths,
        ai_files,
        warnings,
    })
}

async fn generate_with_provider(
    provider: &dyn Provider,
    config: &EffectiveConfig,
    diff_files: &[DiffFile],
    deadline: Instant,
) -> CoreResult<String> {
    let total_tokens: usize = diff_files.iter().map(|file| file.token_estimate).sum();

    if total_tokens <= config.max_input_tokens as usize {
        let diff_text = diff_files_to_string(diff_files);
        let system_prompt = commit_system_prompt(config);
        let user_prompt = commit_user_prompt(&diff_text, config);
        let request = ProviderRequest {
            max_output_tokens: config.max_output_tokens,
            temperature: config.temperature,
        };

        return call_with_deadline(
            deadline,
            provider.complete(&system_prompt, &user_prompt, request),
        )
        .await;
    }

    summarize_then_commit(provider, config, diff_files, deadline).await
}

async fn summarize_then_commit(
    provider: &dyn Provider,
    config: &EffectiveConfig,
    diff_files: &[DiffFile],
    deadline: Instant,
) -> CoreResult<String> {
    let max_file_tokens = std::cmp::min(config.max_input_tokens as usize, 2000);
    let summary_tokens = std::cmp::min(config.max_output_tokens, 120);
    let concurrency = std::cmp::max(config.summary_concurrency, 1);

    let summary_results = stream::iter(diff_files.iter())
        .map(|file| async move {
            let truncated = truncate_to_tokens(&file.content, max_file_tokens);
            if truncated.trim().is_empty() {
                return (file.path.clone(), None);
            }

            let system_prompt = summary_system_prompt();
            let user_prompt = summary_user_prompt(&file.path, &truncated);
            let request = ProviderRequest {
                max_output_tokens: summary_tokens,
                temperature: config.temperature,
            };

            let result = call_with_deadline(
                deadline,
                provider.complete(&system_prompt, &user_prompt, request),
            )
            .await;

            match result {
                Ok(summary) => (file.path.clone(), Some(summary)),
                Err(err) => {
                    warn!("summary failed for {}: {}", file.path, err);
                    (file.path.clone(), None)
                }
            }
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    let mut combined = Vec::new();
    for (path, summary) in summary_results {
        if let Some(summary) = summary {
            combined.push(format!("{}: {}", path, summary.trim()));
        }
    }

    if combined.is_empty() {
        return Ok(String::new());
    }

    let mut combined_text = combined.join("\n");
    let combined_tokens = estimate_tokens(&combined_text);
    if combined_tokens > config.max_input_tokens as usize {
        combined_text = truncate_to_tokens(&combined_text, config.max_input_tokens as usize);
    }

    let system_prompt = commit_system_prompt(config);
    let user_prompt = commit_user_prompt(&combined_text, config);
    let request = ProviderRequest {
        max_output_tokens: config.max_output_tokens,
        temperature: config.temperature,
    };

    call_with_deadline(
        deadline,
        provider.complete(&system_prompt, &user_prompt, request),
    )
    .await
}

async fn call_with_deadline<F>(deadline: Instant, fut: F) -> CoreResult<String>
where
    F: std::future::Future<Output = CoreResult<String>>,
{
    let now = Instant::now();
    if now >= deadline {
        return Err(CoreError::Timeout(0));
    }
    let remaining = deadline.saturating_duration_since(now);
    match tokio::time::timeout(remaining, fut).await {
        Ok(result) => result,
        Err(_) => Err(CoreError::Timeout(remaining.as_secs())),
    }
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

fn trim_quotes(input: &str) -> String {
    let trimmed = input.trim();
    trimmed
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('`')
        .to_string()
}

fn conventional_regex() -> &'static Regex {
    static RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(feat|fix|build|chore|ci|docs|style|refactor|perf|test)(\([\w./-]+\))?: .+")
            .expect("invalid regex")
    });
    &RE
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn sanitize_message_falls_back_for_invalid_conventional() {
        let config = Config::defaults().resolve().expect("defaults resolve");
        let fallback = "chore: update files";
        let cleaned = sanitize_message("updated stuff", &config, fallback);
        assert_eq!(cleaned, fallback);
    }

    #[test]
    fn sanitize_message_strips_code_fences() {
        let config = Config::defaults().resolve().expect("defaults resolve");
        let fallback = "chore: update files";
        let cleaned = sanitize_message("```feat: add api```", &config, fallback);
        assert_eq!(cleaned, "feat: add api");
    }
}
