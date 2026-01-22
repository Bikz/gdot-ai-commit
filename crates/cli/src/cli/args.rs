use std::path::PathBuf;

use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "goodcommit",
    version,
    about = "Good Commit: fast AI commit messages"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    #[arg(value_name = "message", trailing_var_arg = true)]
    pub(crate) message: Vec<String>,

    #[arg(long)]
    pub(crate) provider: Option<String>,
    #[arg(long)]
    pub(crate) model: Option<String>,
    #[arg(long)]
    pub(crate) openai_mode: Option<String>,
    #[arg(long)]
    pub(crate) openai_base_url: Option<String>,
    #[arg(long)]
    pub(crate) ollama_endpoint: Option<String>,
    #[arg(long)]
    pub(crate) timeout: Option<u64>,
    #[arg(long)]
    pub(crate) max_input_tokens: Option<u32>,
    #[arg(long)]
    pub(crate) max_output_tokens: Option<u32>,
    #[arg(long)]
    pub(crate) max_file_bytes: Option<u64>,
    #[arg(long)]
    pub(crate) max_file_lines: Option<u32>,
    #[arg(long)]
    pub(crate) summary_concurrency: Option<u32>,
    #[arg(long)]
    pub(crate) max_files: Option<u32>,
    #[arg(long)]
    pub(crate) lang: Option<String>,

    #[arg(short = 'l', long, action = ArgAction::SetTrue)]
    pub(crate) local: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) conventional: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) no_conventional: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) one_line: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) no_one_line: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) emoji: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) no_emoji: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) push: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) no_push: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) stage_all: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) no_stage: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) interactive: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) yes: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) dry_run: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) edit: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) no_verify: bool,
    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) skip_verify: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    pub(crate) verbose: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    Config,
    Doctor,
    #[command(alias = "init")]
    Setup,
    Split,
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum HookAction {
    Install,
    Uninstall,
    #[command(hide = true)]
    Run {
        path: PathBuf,
        source: Option<String>,
        sha: Option<String>,
    },
}
