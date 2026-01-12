use crate::config::EffectiveConfig;

pub fn commit_system_prompt(config: &EffectiveConfig) -> String {
    let mut prompt = String::from(
        "You are a Git commit message generator that follows the Conventional Commits specification.\n\n",
    );

    if config.conventional {
        prompt.push_str("TASK: Generate a commit message in Conventional Commits format.\n");
        prompt.push_str("FORMAT: <type>(<scope>): <subject>\n");
        prompt.push_str("<type> MUST be one of: feat, fix, build, chore, ci, docs, style, refactor, perf, test\n");
        prompt.push_str("(<scope>) is optional and should be a short noun.\n");
    } else {
        prompt.push_str("TASK: Generate a concise commit message.\n");
    }

    if config.one_line {
        prompt.push_str("OUTPUT: Single line only. No body.\n");
    } else {
        prompt.push_str("OUTPUT: A short subject line, optional blank line, and short body.\n");
    }

    if config.emoji {
        prompt.push_str(
            "If possible, prefix the subject with a relevant emoji for the change type.\n",
        );
    }

    prompt.push_str("RULES:\n");
    prompt.push_str("- Subject must be imperative, lowercase, and concise (max 50 chars).\n");
    prompt.push_str("- Entire message should be plain text, no markdown.\n");
    prompt.push_str("- Do not wrap in quotes or code fences.\n");
    prompt.push_str("- Respond with only the commit message text.\n");

    prompt
}

pub fn commit_user_prompt(diff: &str, config: &EffectiveConfig) -> String {
    if let Some(lang) = &config.lang {
        format!("Generate the commit message in {lang}.\n\nDiff:\n{diff}")
    } else {
        format!("Generate the commit message from this diff:\n\n{diff}")
    }
}

pub fn summary_system_prompt() -> String {
    "You are a code reviewer summarizing diffs. Summarize the changes briefly and factually.\nRULES:\n- Use short bullet points.\n- Mention files and key changes.\n- No markdown code blocks.\n"
        .to_string()
}

pub fn summary_user_prompt(path: &str, diff: &str) -> String {
    format!("Summarize changes for {path}:\n\n{diff}")
}
