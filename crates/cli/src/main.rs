mod cli;
mod hooks;
mod setup;
mod ui;
mod util;

#[tokio::main]
async fn main() {
    tokio::select! {
        result = cli::run() => {
            if let Err(err) = result {
                ui::error(&format!("{err}"));
                std::process::exit(1);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            ui::warn("cancelled");
            std::process::exit(130);
        }
    }
}
