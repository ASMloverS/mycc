# Task 12: Formatter — comment_style

> Status: ⬜ Not started
> Depends: Task 05
> Output: All comments `#` style, normalized

## Goal

Normalize Python comments:
- All comments start with `#` (Python only has `#` comments)
- `#` + one space: `# comment` not `#comment`
- Inline comment: ≥2 spaces before `#`: `code  # comment`
- Preserve shebang (`#!/usr/bin/env python`) and encoding (`# -*- coding: utf-8 -*-`)

## Context

Python: `#` line comments only. No block comment syntax. Multi-line = consecutive `#` lines. Bad patterns: `#comment` (no space), `code # comment` (1 space).

## Steps

### 1. formatter/comment_style.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_comment_style(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.comment_style == CommentStyle::Preserve {
        return Ok(());
    }
    if source.content.is_empty() {
        return Ok(());
    }

    let had_newline = source.content.ends_with('\n');
    let lines: Vec<String> = source
        .content
        .lines()
        .enumerate()
        .map(|(i, line)| fix_line_comment(line, i))
        .collect();

    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}

fn fix_line_comment(line: &str, line_num: usize) -> String {
    if line_num == 0 && (line.starts_with("#!") || line.contains("-*- coding:")) {
        return line.to_string();
    }

    let comment_pos = find_comment_pos(line);
    match comment_pos {
        None => line.to_string(),
        Some(pos) => {
            let code_part = line[..pos].trim_end();
            let comment_part = normalize_comment(&line[pos..]);
            if code_part.is_empty() {
                let indent_len = line.len() - line.trim_start().len();
                let indent = &line[..indent_len];
                format!("{}{}", indent, comment_part)
            } else {
                format!("{}  {}", code_part, comment_part)
            }
        }
    }
}

fn find_comment_pos(line: &str) -> Option<usize> {
    // Walk chars, track string state, find first # outside string
}

fn normalize_comment(comment: &str) -> String {
    let trimmed = comment.trim_start_matches('#').trim_start();
    if trimmed.is_empty() {
        return "#".to_string();
    }
    format!("# {}", trimmed)
}
```

## Tests

```rust
#[test]
fn add_space_after_hash() {
    let input = "#comment\n";
    let expected = "# comment\n";
}

#[test]
fn preserve_correct_comment() {
    let input = "# comment\n";
    // unchanged
}

#[test]
fn inline_comment_spacing() {
    let input = "x = 1 # inline\n";
    let expected = "x = 1  # inline\n";
}

#[test]
fn preserve_shebang() {
    let input = "#!/usr/bin/env python3\n";
    // unchanged
}

#[test]
fn preserve_encoding_declaration() {
    let input = "# -*- coding: utf-8 -*-\n";
    // unchanged
}

#[test]
fn hash_in_string_not_treated_as_comment() {
    let input = "x = \"# not a comment\"\n";
    // unchanged
}

#[test]
fn empty_comment_stays_hash() {
    let input = "#\n";
    // unchanged
}
```

## Verify

```bash
cargo test -- comment_style
```
