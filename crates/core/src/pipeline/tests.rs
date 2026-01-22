use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::{Config, ConfigPaths};
use crate::git::{GitBackend, GitDiff, GitFileStat};
use crate::ignore::build_ignore_matcher;

use super::context::collect_diff_context;
use super::sanitize::sanitize_message;

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

struct StubGit {
    stats: Vec<GitFileStat>,
    diffs: HashMap<String, String>,
}

impl GitBackend for StubGit {
    fn ensure_git_repo(&self) -> crate::error::CoreResult<()> {
        Ok(())
    }

    fn repo_root(&self) -> crate::error::CoreResult<PathBuf> {
        Ok(PathBuf::from("."))
    }

    fn git_dir(&self) -> crate::error::CoreResult<PathBuf> {
        Ok(PathBuf::from(".git"))
    }

    fn stage_all(&self) -> crate::error::CoreResult<()> {
        Ok(())
    }

    fn stage_interactive(&self) -> crate::error::CoreResult<()> {
        Ok(())
    }

    fn stage_paths(&self, _paths: &[String]) -> crate::error::CoreResult<()> {
        Ok(())
    }

    fn unstage_all(&self) -> crate::error::CoreResult<()> {
        Ok(())
    }

    fn staged_diff(&self) -> crate::error::CoreResult<String> {
        Ok(String::new())
    }

    fn staged_diff_for_path(
        &self,
        path: &str,
        _max_bytes: u64,
    ) -> crate::error::CoreResult<GitDiff> {
        let content = self.diffs.get(path).cloned().unwrap_or_default();
        Ok(GitDiff {
            content,
            truncated: false,
        })
    }

    fn staged_files(&self) -> crate::error::CoreResult<Vec<String>> {
        Ok(self.stats.iter().map(|stat| stat.path.clone()).collect())
    }

    fn staged_numstat(&self) -> crate::error::CoreResult<Vec<GitFileStat>> {
        Ok(self.stats.clone())
    }

    fn working_tree_files(&self) -> crate::error::CoreResult<Vec<String>> {
        Ok(Vec::new())
    }

    fn has_unstaged_changes(&self) -> crate::error::CoreResult<bool> {
        Ok(false)
    }

    fn commit(
        &self,
        _message: &str,
        _edit: bool,
        _no_verify: bool,
    ) -> crate::error::CoreResult<String> {
        Ok(String::new())
    }

    fn push(&self) -> crate::error::CoreResult<String> {
        Ok(String::new())
    }
}

#[test]
fn collect_diff_context_skips_empty_diffs_before_limit() {
    let stats = vec![
        GitFileStat {
            path: "file1.txt".to_string(),
            additions: 1,
            deletions: 1,
            is_binary: false,
        },
        GitFileStat {
            path: "file2.txt".to_string(),
            additions: 1,
            deletions: 1,
            is_binary: false,
        },
        GitFileStat {
            path: "file3.txt".to_string(),
            additions: 1,
            deletions: 1,
            is_binary: false,
        },
    ];

    let mut diffs = HashMap::new();
    diffs.insert("file3.txt".to_string(), "diff --git a b".to_string());

    let git = StubGit { stats, diffs };
    let mut config = Config::defaults();
    config.max_files = Some(1);
    let config = config.resolve().expect("config");

    let paths = ConfigPaths {
        global_config: None,
        repo_config: None,
        global_ignore: PathBuf::from("missing"),
        repo_ignore: None,
    };
    let ignore = build_ignore_matcher(&[], &paths).expect("ignore");

    let context = collect_diff_context(&git, &config, &ignore).expect("context");
    assert_eq!(context.ai_files.len(), 1);
    assert_eq!(context.ai_files[0].path, "file3.txt");
}
