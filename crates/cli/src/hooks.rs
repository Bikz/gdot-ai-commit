use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use goodcommit_core::git::GitBackend;

const HOOK_NAME: &str = "prepare-commit-msg";

pub fn install_hook(git: &impl GitBackend) -> Result<()> {
    let git_dir = git.git_dir()?;
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).context("failed to create hooks directory")?;

    let hook_path = hooks_dir.join(HOOK_NAME);
    let script = "#!/bin/sh\n# goodcommit hook\nexec goodcommit hook run \"$1\" \"$2\" \"$3\"\n";
    fs::write(&hook_path, script).context("failed to write hook")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

pub fn uninstall_hook(git: &impl GitBackend) -> Result<()> {
    let git_dir = git.git_dir()?;
    let hook_path = git_dir.join("hooks").join(HOOK_NAME);
    if hook_path.exists() {
        fs::remove_file(hook_path).context("failed to remove hook")?;
    }
    Ok(())
}

pub fn write_hook_message(path: &Path, message: &str) -> Result<()> {
    fs::write(path, format!("{message}\n")).context("failed to write hook message")
}
