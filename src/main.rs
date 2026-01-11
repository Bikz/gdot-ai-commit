mod cli;
mod config;
mod diff;
mod git;
mod hooks;
mod ignore;
mod prompt;
mod providers;
mod setup;
mod ui;
mod util;

#[tokio::main]
async fn main() {
    if let Err(err) = cli::run().await {
        ui::error(&format!("{err}"));
        std::process::exit(1);
    }
}
