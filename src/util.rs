use is_terminal::IsTerminal;

pub fn is_interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub fn join_message_args(args: &[String]) -> Option<String> {
    if args.is_empty() {
        None
    } else {
        Some(args.join(" "))
    }
}

pub fn trim_quotes(input: &str) -> String {
    let trimmed = input.trim();
    trimmed
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('`')
        .to_string()
}
