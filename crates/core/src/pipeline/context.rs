use crate::config::EffectiveConfig;
use crate::diff::{estimate_tokens, truncate_lines, DiffFile};
use crate::error::CoreResult;
use crate::git::GitBackend;
use crate::ignore::IgnoreMatcher;

pub(super) struct DiffContext {
    pub(super) all_paths: Vec<String>,
    pub(super) ai_files: Vec<DiffFile>,
    pub(super) warnings: Vec<String>,
}

pub(super) fn collect_diff_context(
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
    let mut hit_limit = false;

    for stat in stats {
        if ai_files.len() >= config.max_files {
            hit_limit = true;
            break;
        }
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

    if hit_limit {
        warnings.push(format!(
            "only first {} files used for AI summary",
            config.max_files
        ));
    }

    Ok(DiffContext {
        all_paths,
        ai_files,
        warnings,
    })
}
