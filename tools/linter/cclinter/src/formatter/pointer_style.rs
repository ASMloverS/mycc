use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::{FormatConfig, PointerAlignment};
use regex::Regex;
use std::sync::LazyLock;

const TYPE_KEYWORDS: &str = r"(?i)\b(int|char|void|long|short|float|double|unsigned|signed|const|volatile|static|extern|struct|enum|union|bool|auto|register|restrict|inline|size_t|ssize_t|uint8_t|uint16_t|uint32_t|uint64_t|int8_t|int16_t|int32_t|int64_t|uintptr_t|intptr_t|wchar_t|ptrdiff_t|FILE)";

static NORMALIZE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"{}(\*+)(\w)", TYPE_KEYWORDS)).unwrap()
});

static LEFT_ALIGN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"{}\s+(\*+)\s*(\w)", TYPE_KEYWORDS)).unwrap()
});

static RIGHT_ALIGN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"{}(\*+)\s+(\w)", TYPE_KEYWORDS)).unwrap()
});

pub fn fix_pointer_style(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let had_newline = source.content.ends_with('\n');
    let mut in_block_comment = false;
    let lines: Vec<String> = source
        .content
        .lines()
        .map(|line| {
            let result = if in_block_comment {
                line.to_string()
            } else {
                process_line(line, &config.pointer_alignment)
            };
            in_block_comment = update_block_comment_state(in_block_comment, line);
            result
        })
        .collect();
    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}

fn process_line(line: &str, alignment: &PointerAlignment) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return line.to_string();
    }
    let parts = split_outside_strings(line);
    let processed: Vec<String> = parts
        .into_iter()
        .map(|part| {
            if part.starts_with('"') || part.starts_with('\'') {
                return part;
            }
            let segments = split_outside_inline_block_comments(&part);
            let transformed: Vec<String> = segments
                .into_iter()
                .map(|(is_comment, text)| {
                    if is_comment {
                        return text;
                    }
                    let normalized = NORMALIZE_RE.replace_all(&text, "$1$2 $3").to_string();
                    match alignment {
                        PointerAlignment::Left => {
                            LEFT_ALIGN_RE.replace_all(&normalized, "$1$2 $3").to_string()
                        }
                        PointerAlignment::Right => {
                            RIGHT_ALIGN_RE.replace_all(&normalized, "$1 $2$3").to_string()
                        }
                    }
                })
                .collect();
            transformed.join("")
        })
        .collect();
    processed.join("")
}

fn split_outside_inline_block_comments(s: &str) -> Vec<(bool, String)> {
    let mut segments: Vec<(bool, String)> = Vec::new();
    let mut current = String::new();
    let mut in_comment = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !in_comment && i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            if !current.is_empty() {
                segments.push((false, current.clone()));
                current.clear();
            }
            in_comment = true;
            current.push('/');
            current.push('*');
            i += 2;
        } else if in_comment && i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
            current.push('*');
            current.push('/');
            segments.push((true, current.clone()));
            current.clear();
            in_comment = false;
            i += 2;
        } else {
            current.push(chars[i]);
            i += 1;
        }
    }
    if !current.is_empty() {
        segments.push((in_comment, current));
    }
    segments
}

fn update_block_comment_state(in_block: bool, line: &str) -> bool {
    let mut state = in_block;
    let mut in_string = false;
    let mut escape_next = false;
    let mut string_delim = ' ';
    let mut prev = '\0';
    for ch in line.chars() {
        if escape_next {
            escape_next = false;
            prev = ch;
            continue;
        }
        if ch == '\\' && in_string {
            escape_next = true;
            prev = ch;
            continue;
        }
        if in_string {
            if ch == string_delim {
                in_string = false;
            }
            prev = ch;
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_delim = ch;
            prev = ch;
            continue;
        }
        if !state && prev == '/' && ch == '*' {
            state = true;
        } else if state && prev == '*' && ch == '/' {
            state = false;
        }
        prev = ch;
    }
    state
}
