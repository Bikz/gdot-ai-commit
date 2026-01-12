#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
    pub truncated: bool,
    pub additions: u32,
    pub deletions: u32,
    pub token_estimate: usize,
}

pub fn diff_files_to_string(files: &[DiffFile]) -> String {
    files
        .iter()
        .map(|file| file.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn estimate_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    ((chars as f32) / 4.0).ceil() as usize
}

pub fn truncate_lines(text: &str, max_lines: u32) -> (String, bool) {
    if max_lines == 0 {
        return (String::new(), !text.trim().is_empty());
    }

    let mut buffer = String::new();
    let mut count = 0u32;

    for line in text.lines() {
        if count >= max_lines {
            return (buffer.trim_end().to_string(), true);
        }
        buffer.push_str(line);
        buffer.push('\n');
        count += 1;
    }

    (buffer.trim_end().to_string(), false)
}

pub fn truncate_to_tokens(text: &str, max_tokens: usize) -> String {
    let mut buffer = String::new();
    let mut count = 0usize;

    for line in text.lines() {
        let line_tokens = estimate_tokens(line);
        if count + line_tokens > max_tokens {
            break;
        }
        buffer.push_str(line);
        buffer.push('\n');
        count += line_tokens;
    }

    buffer.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_lines_limits_output() {
        let input = "one\ntwo\nthree\n";
        let (out, truncated) = truncate_lines(input, 2);
        assert_eq!(out, "one\ntwo");
        assert!(truncated);
    }

    #[test]
    fn truncate_lines_no_truncation() {
        let input = "one\ntwo\n";
        let (out, truncated) = truncate_lines(input, 3);
        assert_eq!(out, "one\ntwo");
        assert!(!truncated);
    }
}
