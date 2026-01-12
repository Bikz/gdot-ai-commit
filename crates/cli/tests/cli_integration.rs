use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let temp = TempDir::new().expect("tempdir");
    run_git(temp.path(), &["init"]);
    run_git(temp.path(), &["config", "user.name", "Test User"]);
    run_git(temp.path(), &["config", "user.email", "test@example.com"]);
    run_git(temp.path(), &["config", "commit.gpgsign", "false"]);
    temp
}

fn run_git(dir: &Path, args: &[&str]) -> String {
    let output = StdCommand::new("git")
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_PAGER", "cat")
        .args(args)
        .output()
        .expect("run git");

    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    combined.trim().to_string()
}

#[test]
fn dry_run_with_message() {
    let repo = init_repo();
    fs::write(repo.path().join("README.md"), "hello\n").expect("write file");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("goodcommit"));
    cmd.current_dir(repo.path())
        .arg("--dry-run")
        .arg("chore: init");

    cmd.assert()
        .success()
        .stdout(contains("commit message preview"))
        .stdout(contains("dry run enabled"));
}

#[test]
fn commit_with_message() {
    let repo = init_repo();
    fs::write(repo.path().join("README.md"), "hello\n").expect("write file");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("goodcommit"));
    cmd.current_dir(repo.path())
        .arg("--no-push")
        .arg("--yes")
        .arg("chore: init");

    cmd.assert().success();

    let subject = run_git(repo.path(), &["log", "-1", "--pretty=%s"]);
    assert_eq!(subject, "chore: init");

    let status = run_git(repo.path(), &["status", "--porcelain"]);
    assert!(status.is_empty(), "expected clean repo, got: {status}");
}

#[test]
fn clean_tree_message_when_no_changes() {
    let repo = init_repo();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("goodcommit"));
    cmd.current_dir(repo.path());

    cmd.assert()
        .success()
        .stdout(contains("working tree clean"));
}

#[test]
fn setup_requires_interactive_terminal() {
    let repo = init_repo();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("goodcommit"));
    cmd.current_dir(repo.path()).arg("setup");

    cmd.assert()
        .failure()
        .stderr(contains("setup requires an interactive terminal"));
}
