use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use crate::error::{CoreError, CoreResult};

#[derive(Debug, Clone)]
pub struct GitFileStat {
    pub path: String,
    pub additions: u32,
    pub deletions: u32,
    pub is_binary: bool,
}

#[derive(Debug, Clone)]
pub struct GitDiff {
    pub content: String,
    pub truncated: bool,
}

#[allow(clippy::missing_errors_doc)]
pub trait GitBackend {
    fn ensure_git_repo(&self) -> CoreResult<()>;
    fn repo_root(&self) -> CoreResult<PathBuf>;
    fn git_dir(&self) -> CoreResult<PathBuf>;
    fn stage_all(&self) -> CoreResult<()>;
    fn stage_interactive(&self) -> CoreResult<()>;
    fn stage_paths(&self, paths: &[String]) -> CoreResult<()>;
    fn unstage_all(&self) -> CoreResult<()>;
    fn staged_diff(&self) -> CoreResult<String>;
    fn staged_diff_for_path(&self, path: &str, max_bytes: u64) -> CoreResult<GitDiff>;
    fn staged_files(&self) -> CoreResult<Vec<String>>;
    fn staged_numstat(&self) -> CoreResult<Vec<GitFileStat>>;
    fn working_tree_files(&self) -> CoreResult<Vec<String>>;
    fn has_unstaged_changes(&self) -> CoreResult<bool>;
    fn commit(&self, message: &str, edit: bool, no_verify: bool) -> CoreResult<String>;
    fn push(&self) -> CoreResult<String>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemGit;

impl SystemGit {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl GitBackend for SystemGit {
    fn ensure_git_repo(&self) -> CoreResult<()> {
        run_git(["rev-parse", "--is-inside-work-tree"])
            .map(|_| ())
            .map_err(|err| CoreError::Git(format!("not inside a git repository: {err}")))
    }

    fn repo_root(&self) -> CoreResult<PathBuf> {
        let output = run_git(["rev-parse", "--show-toplevel"])?;
        let root = String::from_utf8(output.stdout)?.trim().to_string();
        if root.is_empty() {
            return Err(CoreError::Git("not inside a git repository".to_string()));
        }
        Ok(PathBuf::from(root))
    }

    fn git_dir(&self) -> CoreResult<PathBuf> {
        let output = run_git(["rev-parse", "--git-dir"])?;
        let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
        if git_dir.is_empty() {
            return Err(CoreError::Git(
                "unable to locate .git directory".to_string(),
            ));
        }

        let path = PathBuf::from(git_dir);
        if path.is_absolute() {
            Ok(path)
        } else {
            let root = self.repo_root()?;
            Ok(root.join(path))
        }
    }

    fn stage_all(&self) -> CoreResult<()> {
        run_git_status(["add", "."])
            .map_err(|err| CoreError::Git(format!("failed to stage files: {err}")))
    }

    fn stage_interactive(&self) -> CoreResult<()> {
        run_git_status_stream(["add", "-p"])
            .map_err(|err| CoreError::Git(format!("interactive staging failed: {err}")))
    }

    fn stage_paths(&self, paths: &[String]) -> CoreResult<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut args: Vec<std::ffi::OsString> = Vec::with_capacity(paths.len() + 2);
        args.push("add".into());
        args.push("--".into());
        for path in paths {
            args.push(path.into());
        }

        run_git_status(args).map_err(|err| CoreError::Git(format!("failed to stage files: {err}")))
    }

    fn unstage_all(&self) -> CoreResult<()> {
        run_git_status(["reset", "-q"])
            .map_err(|err| CoreError::Git(format!("failed to unstage files: {err}")))
    }

    fn staged_diff(&self) -> CoreResult<String> {
        let output = run_git(["diff", "--staged"])?;
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    fn staged_diff_for_path(&self, path: &str, max_bytes: u64) -> CoreResult<GitDiff> {
        let args = [
            "diff",
            "--staged",
            "--no-color",
            "--no-ext-diff",
            "--",
            path,
        ];
        let (content, truncated) = run_git_capture_limit(&args, max_bytes)?;
        Ok(GitDiff { content, truncated })
    }

    fn staged_files(&self) -> CoreResult<Vec<String>> {
        let output = run_git(["diff", "--staged", "--name-only", "-z", "--"])?;
        let entries = output
            .stdout
            .split(|byte| *byte == 0)
            .filter(|chunk| !chunk.is_empty())
            .map(|chunk| String::from_utf8_lossy(chunk).trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        Ok(entries)
    }

    fn staged_numstat(&self) -> CoreResult<Vec<GitFileStat>> {
        let output = run_git(["diff", "--staged", "--numstat", "--"])?;
        let stdout = String::from_utf8(output.stdout)?;
        let mut stats = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let mut parts = line.split('\t');
            let additions = parts.next().unwrap_or("0");
            let deletions = parts.next().unwrap_or("0");
            let path = parts.collect::<Vec<_>>().join("\t");
            if path.trim().is_empty() {
                continue;
            }

            let is_binary = additions == "-" || deletions == "-";
            let add_count = additions.parse::<u32>().unwrap_or(0);
            let del_count = deletions.parse::<u32>().unwrap_or(0);

            stats.push(GitFileStat {
                path,
                additions: add_count,
                deletions: del_count,
                is_binary,
            });
        }

        Ok(stats)
    }

    fn working_tree_files(&self) -> CoreResult<Vec<String>> {
        let mut files = Vec::new();

        let output = run_git(["diff", "--name-only", "--"])?;
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(trimmed.to_string());
            }
        }

        let output = run_git(["ls-files", "-o", "--exclude-standard"])?;
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(trimmed.to_string());
            }
        }

        files.sort();
        files.dedup();
        Ok(files)
    }

    fn has_unstaged_changes(&self) -> CoreResult<bool> {
        let output = run_git(["status", "--porcelain"])?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(!stdout.trim().is_empty())
    }

    fn commit(&self, message: &str, edit: bool, no_verify: bool) -> CoreResult<String> {
        let mut args = vec!["commit", "-m", message];
        if edit {
            args.push("-e");
        }
        if no_verify {
            args.push("--no-verify");
        }

        run_git_output(&args)
    }

    fn push(&self) -> CoreResult<String> {
        let upstream = run_git_raw(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|out| out.trim().to_string())
            .filter(|out| !out.is_empty());

        if upstream.is_some() {
            return run_git_output(&["push"]);
        }

        let branch_output = run_git(["rev-parse", "--abbrev-ref", "HEAD"])?;
        let branch = String::from_utf8(branch_output.stdout)?.trim().to_string();

        let remotes_output = run_git(["remote"])?;
        let remotes = String::from_utf8(remotes_output.stdout)?;
        let remote = remotes
            .lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .ok_or_else(|| CoreError::Git("no git remotes found".to_string()))?;

        run_git_output(&["push", &remote, &branch])
    }
}

fn run_git<I, S>(args: I) -> CoreResult<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()
        .map_err(|err| CoreError::Git(format!("failed to run git command: {err}")))?;

    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(CoreError::Git(stderr))
    }
}

fn run_git_raw<I, S>(args: I) -> CoreResult<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()
        .map_err(|err| CoreError::Git(format!("failed to run git command: {err}")))
}

fn run_git_output(args: &[&str]) -> CoreResult<String> {
    let output = run_git(args)?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let combined = format!("{stdout}{stderr}");
    Ok(combined.trim().to_string())
}

fn run_git_status<I, S>(args: I) -> CoreResult<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args(args)
        .status()
        .map_err(|err| CoreError::Git(format!("failed to run git command: {err}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CoreError::Git("git command failed".to_string()))
    }
}

fn run_git_status_stream<I, S>(args: I) -> CoreResult<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args(args)
        .status()
        .map_err(|err| CoreError::Git(format!("failed to run git command: {err}")))?;

    if status.success() {
        Ok(())
    } else {
        Err(CoreError::Git("git command failed".to_string()))
    }
}

fn run_git_capture_limit(args: &[&str], max_bytes: u64) -> CoreResult<(String, bool)> {
    if max_bytes == 0 {
        return Ok((String::new(), true));
    }

    let mut child = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| CoreError::Git(format!("failed to run git command: {err}")))?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| CoreError::Git("failed to capture git stdout".to_string()))?;

    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| CoreError::Git("failed to capture git stderr".to_string()))?;

    let mut buffer = Vec::new();
    let mut truncated = false;
    let mut total = 0u64;
    let mut chunk = [0u8; 8192];

    loop {
        let read = stdout.read(&mut chunk)?;
        if read == 0 {
            break;
        }

        let remaining = usize::try_from(max_bytes.saturating_sub(total)).unwrap_or(usize::MAX);
        if remaining == 0 {
            truncated = true;
            break;
        }

        let to_take = std::cmp::min(remaining, read);
        buffer.extend_from_slice(&chunk[..to_take]);
        total += to_take as u64;

        if to_take < read {
            truncated = true;
            break;
        }
    }

    if truncated {
        let _ = child.kill();
    }

    let mut stderr_buf = String::new();
    let _ = stderr.read_to_string(&mut stderr_buf);

    let status = child.wait()?;
    if !status.success() && !truncated {
        let message = stderr_buf.trim().to_string();
        return Err(CoreError::Git(message));
    }

    let content = String::from_utf8(buffer)?.trim().to_string();
    Ok((content, truncated))
}
