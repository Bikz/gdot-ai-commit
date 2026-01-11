use std::path::PathBuf;
use std::process::{Command, Output};

use anyhow::{anyhow, Context, Result};

pub fn repo_root() -> Result<PathBuf> {
    let output = run_git(["rev-parse", "--show-toplevel"])?;
    let root = String::from_utf8(output.stdout)?.trim().to_string();
    if root.is_empty() {
        return Err(anyhow!("not inside a git repository"));
    }
    Ok(PathBuf::from(root))
}

pub fn git_dir() -> Result<PathBuf> {
    let output = run_git(["rev-parse", "--git-dir"])?;
    let git_dir = String::from_utf8(output.stdout)?.trim().to_string();
    if git_dir.is_empty() {
        return Err(anyhow!("unable to locate .git directory"));
    }

    let path = PathBuf::from(git_dir);
    if path.is_absolute() {
        Ok(path)
    } else {
        let root = repo_root()?;
        Ok(root.join(path))
    }
}

pub fn ensure_git_repo() -> Result<()> {
    run_git(["rev-parse", "--is-inside-work-tree"]).context("not inside a git repository")?;
    Ok(())
}

pub fn stage_all() -> Result<()> {
    run_git_status(["add", "."]).context("failed to stage files")
}

pub fn stage_interactive() -> Result<()> {
    run_git_status_stream(["add", "-p"]).context("interactive staging failed")
}

pub fn staged_diff() -> Result<String> {
    let output = run_git(["diff", "--staged"])?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

pub fn staged_files() -> Result<Vec<String>> {
    let output = run_git(["diff", "--staged", "--name-only"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

pub fn has_unstaged_changes() -> Result<bool> {
    let output = run_git(["status", "--porcelain"])?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(!stdout.trim().is_empty())
}

pub fn commit(message: &str, edit: bool, no_verify: bool) -> Result<String> {
    let mut args = vec!["commit", "-m", message];
    if edit {
        args.push("-e");
    }
    if no_verify {
        args.push("--no-verify");
    }

    let output = run_git_output(&args)?;
    Ok(output)
}

pub fn push() -> Result<String> {
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
        .ok_or_else(|| anyhow!("no git remotes found"))?;

    run_git_output(&["push", &remote, &branch])
}

fn run_git<I, S>(args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let output = Command::new("git")
        .args(args)
        .output()
        .context("failed to run git command")?;

    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!(stderr.trim().to_string()))
    }
}

fn run_git_raw<I, S>(args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new("git")
        .args(args)
        .output()
        .context("failed to run git command")
}

fn run_git_output(args: &[&str]) -> Result<String> {
    let output = run_git(args)?;
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    let combined = format!("{}{}", stdout, stderr);
    Ok(combined.trim().to_string())
}

fn run_git_status<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("git")
        .args(args)
        .status()
        .context("failed to run git command")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("git command failed"))
    }
}

fn run_git_status_stream<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let status = Command::new("git")
        .args(args)
        .status()
        .context("failed to run git command")?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("git command failed"))
    }
}
