use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;
use regex::Regex;
use std::sync::LazyLock;

static STRUCT_OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:typedef\s+)?struct\b.*\{").unwrap());

static ENUM_OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*(?:typedef\s+)?enum\b.*\{").unwrap());

static FIELD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(\S+(?:\s+\S+)*?)\s+(\w+(?:\[[^\]]*\])?)(\s*;.*)$").unwrap());

static BLOCK_COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/\*.*?\*/").unwrap());

static ENUM_MEMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(\w+)(\s*=\s*)(.+)$").unwrap());

static ENUM_PLAIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)(\w+)\s*(,.*)$").unwrap());

pub fn fix_alignment(
    source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
    let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    let mut i = 0;
    while i < result.len() {
        let line = &result[i];
        if STRUCT_OPEN_RE.is_match(line) {
            i = process_block(&mut result, i, BlockKind::Struct);
        } else if ENUM_OPEN_RE.is_match(line) {
            i = process_block(&mut result, i, BlockKind::Enum);
        } else {
            i += 1;
        }
    }
    let joined = result.join("\n");
    source.content = if had_newline {
        format!("{}\n", joined)
    } else {
        joined
    };
    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum BlockKind {
    Struct,
    Enum,
}

fn process_block(lines: &mut [String], start: usize, kind: BlockKind) -> usize {
    let mut depth = 1;
    let mut i = start + 1;
    let mut field_ranges: Vec<(usize, ParsedField)> = Vec::new();
    while i < lines.len() && depth > 0 {
        let line = &lines[i];
        let trimmed = line.trim();
        let open = count_braces(trimmed, '{');
        let close = count_braces(trimmed, '}');
        if open > 0 && close == 0 {
            depth += open as i32;
            if depth == 2 {
                let inner_kind = if trimmed.contains("enum") {
                    BlockKind::Enum
                } else {
                    BlockKind::Struct
                };
                let next = process_block(lines, i, inner_kind);
                depth -= 1;
                i = next;
                continue;
            }
        } else if close > 0 && open == 0 {
            depth -= close as i32;
            if depth <= 0 {
                i += 1;
                break;
            }
        } else if open > 0 && close > 0 {
            depth += open as i32;
            depth -= close as i32;
            if depth <= 0 {
                i += 1;
                break;
            }
        }
        if depth == 1
            && !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("/*")
            && !(trimmed.starts_with("* ") || trimmed == "*")
            && !trimmed.starts_with("*/")
        {
            match kind {
                BlockKind::Struct => {
                    if let Some(parsed) = parse_struct_field(&lines[i]) {
                        field_ranges.push((i, parsed));
                    }
                }
                BlockKind::Enum => {
                    if let Some(parsed) = parse_enum_member(&lines[i]) {
                        field_ranges.push((i, parsed));
                    }
                }
            }
        }
        i += 1;
    }
    if field_ranges.len() >= 2 {
        match kind {
            BlockKind::Struct => align_struct_fields(lines, &field_ranges),
            BlockKind::Enum => align_enum_members(lines, &field_ranges),
        }
    }
    i
}

fn count_braces(s: &str, brace: char) -> usize {
    let parts = split_outside_strings(s);
    let mut count = 0;
    for part in &parts {
        if part.starts_with('"') || part.starts_with('\'') {
            continue;
        }
        let cleaned = BLOCK_COMMENT_RE.replace_all(part, "");
        count += cleaned.matches(brace).count();
    }
    count
}

struct ParsedField {
    indent: String,
    prefix: String,
    name: String,
    suffix: String,
}

fn strip_trailing_comment(s: &str) -> (String, Option<String>) {
    let bytes = s.as_bytes();
    let mut in_string = false;
    let mut escape_next = false;
    let mut delim = b' ';
    let mut comment_start = None;
    for i in 0..bytes.len() {
        let ch = bytes[i];
        if escape_next {
            escape_next = false;
            continue;
        }
        if in_string {
            if ch == b'\\' {
                escape_next = true;
            } else if ch == delim {
                in_string = false;
            }
            continue;
        }
        if ch == b'"' || ch == b'\'' {
            in_string = true;
            delim = ch;
            continue;
        }
        if ch == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' || bytes[i + 1] == b'*' {
                comment_start = Some(i);
                break;
            }
        }
    }
    match comment_start {
        Some(pos) => (s[..pos].to_string(), Some(s[pos..].to_string())),
        None => (s.to_string(), None),
    }
}

fn parse_struct_field(line: &str) -> Option<ParsedField> {
    let (code, comment) = strip_trailing_comment(line);
    let trimmed = code.trim_end();
    if !trimmed.ends_with(';') {
        return None;
    }
    if let Some(eq_pos) = trimmed.find('=') {
        if eq_pos < trimmed.len() - 1 {
            return None;
        }
    }
    if trimmed.contains(" : ") {
        return None;
    }
    let caps = FIELD_RE.captures(&code)?;
    let indent = caps.get(1).unwrap().as_str().to_string();
    let type_part = caps.get(2).unwrap().as_str().to_string();
    let name = caps.get(3).unwrap().as_str().to_string();
    let rest = caps.get(4).unwrap().as_str().to_string();
    let suffix = match comment {
        Some(c) => format!("{}{}", rest, c),
        None => rest,
    };
    Some(ParsedField {
        indent,
        prefix: type_part,
        name,
        suffix,
    })
}

fn parse_enum_member(line: &str) -> Option<ParsedField> {
    let (code, comment) = strip_trailing_comment(line);
    if let Some(caps) = ENUM_MEMBER_RE.captures(&code) {
        let indent = caps.get(1).unwrap().as_str().to_string();
        let name = caps.get(2).unwrap().as_str().to_string();
        let eq_part = caps.get(3).unwrap().as_str().trim().to_string();
        let value = caps.get(4).unwrap().as_str().to_string();
        let suffix = match comment {
            Some(c) => format!(" {} {} {}", eq_part, value.trim(), c),
            None => format!(" {} {}", eq_part, value.trim()),
        };
        return Some(ParsedField {
            indent,
            prefix: String::new(),
            name,
            suffix,
        });
    }
    if let Some(caps) = ENUM_PLAIN_RE.captures(&code) {
        let indent = caps.get(1).unwrap().as_str().to_string();
        let name = caps.get(2).unwrap().as_str().to_string();
        let rest = caps.get(3).unwrap().as_str().to_string();
        let suffix = match comment {
            Some(c) => format!("{}{}", rest, c),
            None => rest,
        };
        return Some(ParsedField {
            indent,
            prefix: String::new(),
            name,
            suffix,
        });
    }
    None
}

fn align_struct_fields(lines: &mut [String], fields: &[(usize, ParsedField)]) {
    let max_type_len = fields
        .iter()
        .map(|(_, f)| {
            let clean = f.prefix.trim_end();
            clean.len()
        })
        .max()
        .unwrap_or(0);
    for (idx, parsed) in fields {
        let clean_type = parsed.prefix.trim_end();
        let needed = max_type_len.saturating_sub(clean_type.len());
        let padding = " ".repeat(needed);
        let new_line = format!(
            "{}{}{} {}{}",
            parsed.indent, clean_type, padding, parsed.name, parsed.suffix
        );
        lines[*idx] = new_line;
    }
}

fn align_enum_members(lines: &mut [String], fields: &[(usize, ParsedField)]) {
    let has_equals = fields.iter().any(|(_, f)| f.suffix.trim_start().starts_with('='));
    if !has_equals {
        return;
    }
    let eq_fields: Vec<(usize, &ParsedField)> = fields
        .iter()
        .filter(|(_, f)| f.suffix.trim_start().starts_with('='))
        .map(|(idx, f)| (*idx, f))
        .collect();
    let max_name_len = eq_fields
        .iter()
        .map(|(_, f)| f.name.len())
        .max()
        .unwrap_or(0);
    for (idx, parsed) in &eq_fields {
        let needed = max_name_len.saturating_sub(parsed.name.len());
        let padding = " ".repeat(needed);
        let new_line = format!(
            "{}{}{}{}",
            parsed.indent, parsed.name, padding, parsed.suffix
        );
        lines[*idx] = new_line;
    }
}
