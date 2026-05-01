# Task 13: Formatter — line_length wrap (token-aware)

> Status: ⬜ Not started
> Depends: Task 05
> Output: long lines wrap at token boundary

## Goal

Enforce line length limit:
- Lines > `column_limit` (default 120) wrap at token boundary
- Skip: URLs, string literals, comments
- Wrapped lines use bracket align or hanging indent

## PEP 8 Rules

- PEP 8 suggests 79, accepts 99
- Project default 120, configurable
- Prefer bracket implicit continuation

## Steps

### 1. formatter/line_length.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_line_length(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let limit = config.column_limit;
    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();

    let result: Vec<String> = lines
        .iter()
        .flat_map(|line| {
            if line.chars().count() <= limit {
                return vec![line.to_string()];
            }
            if should_skip_line(line) {
                return vec![line.to_string()];
            }
            wrap_line(line, limit, config.indent_width)
        })
        .collect();

    let joined = result.join("\n");
    source.content = if had_newline && !joined.is_empty() {
        format!("{}\n", joined)
    } else {
        joined
    };
    Ok(())
}

fn should_skip_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('#')
        || trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
}

fn wrap_line(line: &str, limit: usize, indent_width: usize) -> Vec<String> {
    let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let cont_indent = format!("{}{}", leading_ws, " ".repeat(indent_width));
    wrap_at_token_boundary(line, limit, &cont_indent)
}
```

### 2. Token boundary wrap

```rust
fn wrap_at_token_boundary(line: &str, limit: usize, cont_indent: &str) -> Vec<String> {
    // tokenizer splits line → tokens
    // best break point within limit:
    //   priority: comma > bracket > operator > space
    // insert newline + cont_indent at break
}
```

## Tests

```rust
#[test]
fn no_wrap_short_line() {
    let input = "x = 1\n";
    let config = FormatConfig { column_limit: 120, ..Default::default() };
    // unchanged
}

#[test]
fn wrap_long_function_call() {
    let input = "result = some_function(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10)\n";
    // wrap after comma
}

#[test]
fn skip_comment_lines() {
    let long_comment = format!("# {}{}\n", "x".repeat(200), "\n");
    // no wrap
}

#[test]
fn wrap_preserves_indent() {
    // hanging indent correct inside fn body
}

#[test]
fn idempotent_wrapping() {
    // re-format → no change
}
```

## Verify

```bash
cargo test -- line_length
```
