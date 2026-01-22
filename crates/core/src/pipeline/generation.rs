use std::time::Instant;

use futures::{stream, StreamExt};
use tracing::{debug, instrument, warn};

use crate::config::EffectiveConfig;
use crate::diff::{diff_files_to_string, estimate_tokens, truncate_to_tokens, DiffFile};
use crate::error::{CoreError, CoreResult};
use crate::prompt::{
    commit_system_prompt, commit_user_prompt, summary_system_prompt, summary_user_prompt,
};
use crate::providers::{Provider, ProviderRequest};

#[instrument(level = "debug", skip(provider, config, diff_files, deadline))]
pub(super) async fn generate_with_provider(
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

#[instrument(level = "debug", skip(provider, config, diff_files, deadline))]
pub(super) async fn summarize_then_commit(
    provider: &dyn Provider,
    config: &EffectiveConfig,
    diff_files: &[DiffFile],
    deadline: Instant,
) -> CoreResult<String> {
    let start = Instant::now();
    let max_file_tokens = std::cmp::min(config.max_input_tokens as usize, 2000);
    let summary_tokens = config.max_output_tokens;
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
                    warn!(path = %file.path, "summary failed: {err}");
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

    let message = call_with_deadline(
        deadline,
        provider.complete(&system_prompt, &user_prompt, request),
    )
    .await;

    debug!(
        elapsed_ms = start.elapsed().as_millis(),
        "summary pipeline complete"
    );

    message
}

pub(super) async fn call_with_deadline<F>(deadline: Instant, fut: F) -> CoreResult<String>
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
