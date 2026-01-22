use std::env;

use tracing_subscriber::EnvFilter;

pub(crate) fn init_tracing(verbose: bool) {
    let default_filter = if verbose {
        "goodcommit=debug,goodcommit_core=debug"
    } else {
        "goodcommit=info,goodcommit_core=info"
    };

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));

    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false);

    if json_logging_enabled() {
        builder
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .init();
    } else {
        builder.init();
    }
}

fn json_logging_enabled() -> bool {
    if let Ok(format) = env::var("GOODCOMMIT_LOG_FORMAT") {
        if format.eq_ignore_ascii_case("json") {
            return true;
        }
    }

    env::var("GOODCOMMIT_LOG_JSON")
        .ok()
        .as_deref()
        .map(parse_bool)
        .unwrap_or(false)
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
