use regex::Regex;
use std::sync::LazyLock;

use crate::config::EffectiveConfig;

pub(super) fn sanitize_message(raw: &str, config: &EffectiveConfig, fallback: &str) -> String {
    let cleaned = trim_quotes(raw);
    let mut message = cleaned.trim().to_string();

    if config.one_line {
        message = message.lines().next().unwrap_or("").trim().to_string();
    }

    message = message.replace("```", "").replace('`', "");

    if config.conventional {
        let re = conventional_regex();
        let first_line = message.lines().next().unwrap_or("").trim();
        if !re.is_match(first_line) {
            if let Some(found) = cleaned.lines().find(|line| re.is_match(line.trim())) {
                message = found.trim().to_string();
            } else {
                message = fallback.to_string();
            }
        }
    }

    if message.is_empty() {
        fallback.to_string()
    } else {
        message
    }
}

fn trim_quotes(input: &str) -> String {
    let trimmed = input.trim();
    trimmed
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('`')
        .to_string()
}

fn conventional_regex() -> &'static Regex {
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(feat|fix|build|chore|ci|docs|style|refactor|perf|test)(\([\w./-]+\))?: .+")
            .expect("invalid regex")
    });
    &RE
}
