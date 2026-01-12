use std::fs;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::ConfigPaths;
use crate::error::{CoreError, CoreResult};

pub struct IgnoreMatcher {
    globset: GlobSet,
}

impl IgnoreMatcher {
    pub fn is_ignored(&self, path: &str) -> bool {
        self.globset.is_match(path)
    }
}

pub fn build_ignore_matcher(
    config_patterns: &[String],
    paths: &ConfigPaths,
) -> CoreResult<IgnoreMatcher> {
    let mut patterns = Vec::new();
    patterns.extend(default_patterns());

    if let Some(repo_ignore) = &paths.repo_ignore {
        patterns.extend(read_ignore_file(repo_ignore));
    }

    patterns.extend(read_ignore_file(&paths.global_ignore));

    patterns.extend(config_patterns.iter().cloned());

    let mut builder = GlobSetBuilder::new();
    for pattern in &patterns {
        if pattern.trim().is_empty() {
            continue;
        }
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }

    let globset = builder
        .build()
        .map_err(|err| CoreError::Config(format!("invalid ignore patterns: {err}")))?;

    Ok(IgnoreMatcher { globset })
}

pub fn read_ignore_file(path: &Path) -> Vec<String> {
    if let Ok(content) = fs::read_to_string(path) {
        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.to_string())
            .collect()
    } else {
        Vec::new()
    }
}

pub fn default_patterns() -> Vec<String> {
    vec![
        "node_modules".to_string(),
        "**/node_modules/**".to_string(),
        "dist".to_string(),
        "**/dist/**".to_string(),
        "build".to_string(),
        "**/build/**".to_string(),
        ".next".to_string(),
        "**/.next/**".to_string(),
        ".turbo".to_string(),
        "**/.turbo/**".to_string(),
        ".vite".to_string(),
        "**/.vite/**".to_string(),
        "coverage".to_string(),
        "**/coverage/**".to_string(),
        "*.lock".to_string(),
        "**/*.lock".to_string(),
        "bun.lock".to_string(),
        "bun.lockb".to_string(),
        "package-lock.json".to_string(),
        "pnpm-lock.yaml".to_string(),
        "yarn.lock".to_string(),
        "Pods".to_string(),
        "**/Pods/**".to_string(),
        "*.xcworkspace".to_string(),
        "**/*.xcworkspace/**".to_string(),
        "*.pbxproj".to_string(),
        "**/*.pbxproj".to_string(),
        "*.xcodeproj".to_string(),
        "**/*.xcodeproj/**".to_string(),
        "DerivedData".to_string(),
        "**/DerivedData/**".to_string(),
        "target".to_string(),
        "**/target/**".to_string(),
        "**/*.min.js".to_string(),
        "**/*.min.css".to_string(),
        "**/*.map".to_string(),
    ]
}
