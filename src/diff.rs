use crate::ignore::IgnoreMatcher;

#[derive(Debug, Clone)]
pub struct DiffFile {
    pub path: String,
    pub content: String,
    pub is_binary: bool,
}

pub fn parse_diff(diff: &str) -> Vec<DiffFile> {
    let mut files = Vec::new();
    let mut current: Option<DiffFile> = None;

    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            if let Some(file) = current.take() {
                files.push(file);
            }

            let path = parse_path_from_diff_header(line).unwrap_or_else(|| "unknown".to_string());
            current = Some(DiffFile {
                path,
                content: format!("{line}\n"),
                is_binary: false,
            });
            continue;
        }

        if let Some(file) = current.as_mut() {
            if line.starts_with("Binary files ") || line.contains("GIT binary patch") {
                file.is_binary = true;
            }
            file.content.push_str(line);
            file.content.push('\n');
        }
    }

    if let Some(file) = current.take() {
        files.push(file);
    }

    files
}

fn parse_path_from_diff_header(header: &str) -> Option<String> {
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() >= 4 {
        let path = parts[3];
        return Some(path.trim_start_matches("b/").to_string());
    }
    None
}

pub fn filter_diff_files(files: Vec<DiffFile>, ignore: &IgnoreMatcher) -> Vec<DiffFile> {
    files
        .into_iter()
        .filter(|file| !file.is_binary)
        .filter(|file| !ignore.is_ignored(&file.path))
        .collect()
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
