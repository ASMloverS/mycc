use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;
use regex::Regex;
use std::sync::LazyLock;

static COMPOUND_OP_DATA: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    let ops: &[&str] = &[
        "++", "--", "<<=", ">>=", "==", "!=", "<=", ">=", "&&", "||", "<<", ">>",
        "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
    ];
    ops.iter()
        .map(|op| {
            let pattern = format!(r"(?P<before>\S)\s*{}\s*(?P<after>\S)", regex::escape(op));
            (*op, Regex::new(&pattern).unwrap())
        })
        .collect()
});

static SINGLE_OP_DATA: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    let ops: &[&str] = &["+", "-", "*", "/", "%", "<", ">", "&", "|", "^", "="];
    ops.iter()
        .map(|op| {
            let pattern = format!(r"(?P<before>\S)\s*{}\s*(?P<after>\S)", regex::escape(op));
            (*op, Regex::new(&pattern).unwrap())
        })
        .collect()
});

static COMMA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r",\s*").unwrap());

static FOR_PAREN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(for\s*\()([^)]*)(\))").unwrap());

static SEMI_WS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";\s*").unwrap());

static SPACE_BEFORE_PAREN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\w+)\(").unwrap());

fn is_unary_context(ch: char) -> bool {
    matches!(
        ch,
        '=' | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '<'
            | '>'
            | '&'
            | '|'
            | '^'
            | '!'
            | '('
            | '['
            | ','
            | ';'
            | '{'
            | '}'
            | '?'
            | ':'
            | '~'
    )
}

pub fn fix_spacing(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.spaces_around_operators && !config.space_before_paren {
        return Ok(());
    }
    let had_newline = source.content.ends_with('\n');
    let lines: Vec<String> = source
        .content
        .lines()
        .map(|line| process_line_spacing(line, config))
        .collect();
    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}

fn process_line_spacing(line: &str, config: &FormatConfig) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return line.to_string();
    }
    let parts = split_outside_strings(trimmed);
    let processed: Vec<String> = parts
        .into_iter()
        .map(|part| {
            if part.starts_with('"') || part.starts_with('\'') {
                return part;
            }
            let mut result = apply_operator_spacing(&part);
            if config.space_before_paren {
                result = apply_space_before_paren(&result);
            }
            result
        })
        .collect();
    let mut result = processed.join("");
    result = fix_for_semicolons(&result);
    let orig_prefix_len = line.len() - line.trim_start().len();
    let prefix = &line[..orig_prefix_len];
    format!("{}{}", prefix, result)
}

fn apply_operator_spacing(input: &str) -> String {
    let mut result = input.to_string();
    for (op, re) in COMPOUND_OP_DATA.iter() {
        if *op == "++" || *op == "--" {
            continue;
        }
        loop {
            let mut found = false;
            for cap in re.captures_iter(&result.clone()) {
                let m = cap.get(0).unwrap();
                let full = m.as_str();
                if is_part_of_longer_compound_op(full, op) {
                    continue;
                }
                let before = cap.name("before").unwrap().as_str();
                let after = cap.name("after").unwrap().as_str();
                let replacement_str = format!("{} {} {}", before, op, after);
                if full == replacement_str {
                    continue;
                }
                let new = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    &replacement_str,
                    &result[m.end()..]
                );
                result = new;
                found = true;
                break;
            }
            if !found {
                break;
            }
        }
    }
    for (op, re) in SINGLE_OP_DATA.iter() {
        loop {
            let mut found = false;
            for cap in re.captures_iter(&result.clone()) {
                let m = cap.get(0).unwrap();
                let full = m.as_str();
                if is_substring_of_compound_op(full, op) {
                    continue;
                }
                let before_char = cap
                    .name("before")
                    .unwrap()
                    .as_str()
                    .chars()
                    .next()
                    .unwrap();
                if (*op == "-" || *op == "+") && is_unary_context(before_char) {
                    continue;
                }
                if *op == "&" && is_unary_context(before_char) {
                    continue;
                }
                if *op == "*" && before_char == '(' {
                    continue;
                }
                let before = cap.name("before").unwrap().as_str();
                let after = cap.name("after").unwrap().as_str();
                let replacement_str = format!("{} {} {}", before, op, after);
                if full == replacement_str {
                    continue;
                }
                let new = format!(
                    "{}{}{}",
                    &result[..m.start()],
                    &replacement_str,
                    &result[m.end()..]
                );
                result = new;
                found = true;
                break;
            }
            if !found {
                break;
            }
        }
    }
    result = COMMA_RE.replace_all(&result, ", ").to_string();
    result
}

const COMPOUND_OPS: &[&str] = &[
    "++", "--", "<<=", ">>=", "==", "!=", "<=", ">=", "&&", "||", "<<", ">>",
    "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=",
];

fn is_substring_of_compound_op(full: &str, op: &str) -> bool {
    for cop in COMPOUND_OPS.iter().copied().chain(std::iter::once("->")) {
        if full.contains(cop) && cop != op && cop.contains(op) {
            return true;
        }
    }
    false
}

fn is_part_of_longer_compound_op(full: &str, op: &str) -> bool {
    for lop in COMPOUND_OPS {
        if lop.len() > op.len() && full.contains(*lop) && lop.contains(op) {
            return true;
        }
    }
    false
}

fn fix_for_semicolons(input: &str) -> String {
    FOR_PAREN_RE
        .replace_all(input, |caps: &regex::Captures| {
            let prefix = caps.get(1).unwrap().as_str();
            let inner = caps.get(2).unwrap().as_str();
            let suffix = caps.get(3).unwrap().as_str();
            let fixed_inner = SEMI_WS_RE.replace_all(inner, "; ");
            format!("{}{}{}", prefix, fixed_inner, suffix)
        })
        .to_string()
}

fn apply_space_before_paren(input: &str) -> String {
    SPACE_BEFORE_PAREN_RE
        .replace_all(input, "$1 (")
        .to_string()
}
