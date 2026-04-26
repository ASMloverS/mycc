use crate::common::source::SourceFile;
use crate::config::{FormatConfig, IncludeSorting};
use regex::Regex;
use std::sync::LazyLock;

static INCLUDE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*#\s*include\s+([<"])([^>"]+)[>"]"#).unwrap());

static COND_PP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"^\s*#\s*(if|ifdef|ifndef|elif|else|endif)\b"#).unwrap());

pub fn fix_include_sort(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.include_sorting == IncludeSorting::Disabled {
        return Ok(());
    }
    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();
    if lines.is_empty() {
        return Ok(());
    }
    let first_idx = lines.iter().position(|l| INCLUDE_RE.is_match(l));
    let last_idx = lines.iter().rposition(|l| INCLUDE_RE.is_match(l));
    if let (Some(first), Some(last)) = (first_idx, last_idx) {
        for line in lines.iter().take(last + 1).skip(first + 1) {
            if COND_PP_RE.is_match(line) {
                return Ok(());
            }
            if !line.trim().is_empty() && !INCLUDE_RE.is_match(line) {
                return Ok(());
            }
        }
    }
    let stem = source
        .path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut corresponding: Vec<(String, String)> = vec![];
    let mut system: Vec<(String, String)> = vec![];
    let mut project: Vec<(String, String)> = vec![];
    let mut pre_include: Vec<String> = vec![];
    let mut post_include: Vec<String> = vec![];
    let mut in_includes = false;
    let mut past_includes = false;
    for line in &lines {
        if let Some(caps) = INCLUDE_RE.captures(line) {
            let delimiter = &caps[1];
            let header = &caps[2];
            let entry = (header.to_string(), line.trim().to_string());
            in_includes = true;
            if delimiter == "<" {
                system.push(entry);
            } else {
                let header_stem = std::path::Path::new(header)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if header_stem == stem {
                    corresponding.push(entry);
                } else {
                    project.push(entry);
                }
            }
        } else if in_includes && line.trim().is_empty() && !past_includes {
            continue;
        } else if in_includes {
            past_includes = true;
            post_include.push(line.to_string());
        } else {
            pre_include.push(line.to_string());
        }
    }
    if corresponding.is_empty() && system.is_empty() && project.is_empty() {
        return Ok(());
    }
    corresponding.sort_by(|a, b| a.0.cmp(&b.0));
    system.sort_by(|a, b| a.0.cmp(&b.0));
    project.sort_by(|a, b| a.0.cmp(&b.0));
    let mut result = pre_include;
    for (_, line) in &corresponding {
        result.push(line.clone());
    }
    if !corresponding.is_empty() && !system.is_empty() {
        result.push(String::new());
    }
    for (_, line) in &system {
        result.push(line.clone());
    }
    if (!corresponding.is_empty() || !system.is_empty()) && !project.is_empty() {
        result.push(String::new());
    }
    for (_, line) in &project {
        result.push(line.clone());
    }
    if !post_include.is_empty() {
        result.push(String::new());
        result.extend(post_include);
    }
    let joined = result.join("\n");
    source.content = if had_newline && !joined.is_empty() {
        format!("{}\n", joined)
    } else {
        joined
    };
    Ok(())
}
