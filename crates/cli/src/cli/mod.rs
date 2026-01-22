use anyhow::Result;
use clap::Parser;
use goodcommit_core::git::GitBackend;

use crate::{hooks, setup, ui};

mod args;
mod commit;
mod config;
mod doctor;
mod tracing;

pub(crate) use args::{Cli, Commands, HookAction};

pub async fn run() -> Result<()> {
    let mut cli = Cli::parse();
    tracing::init_tracing(cli.verbose);

    let command = cli.command.take();

    match command {
        Some(Commands::Setup) => {
            setup::run_setup()?;
            ui::success("setup complete");
            return Ok(());
        }
        Some(Commands::Config) => {
            doctor::run_config(&cli)?;
            return Ok(());
        }
        Some(Commands::Doctor) => {
            doctor::run_doctor(&cli)?;
            return Ok(());
        }
        Some(Commands::Split) => {
            commit::run_split(cli).await?;
            return Ok(());
        }
        Some(Commands::Hook { action }) => match action {
            HookAction::Install => {
                let git = goodcommit_core::git::SystemGit::new();
                git.ensure_git_repo()?;
                hooks::install_hook(&git)?;
                ui::success("hook installed");
                return Ok(());
            }
            HookAction::Uninstall => {
                let git = goodcommit_core::git::SystemGit::new();
                git.ensure_git_repo()?;
                hooks::uninstall_hook(&git)?;
                ui::success("hook removed");
                return Ok(());
            }
            HookAction::Run { path, source, .. } => {
                commit::run_hook(path, source, cli).await?;
                return Ok(());
            }
        },
        None => {}
    }

    commit::run_commit(cli).await
}
