# Task 07: Formatter — Trailing Whitespace Removal

> Status: ⬜ Not started
> Depends on: Task 05
> Output: All trailing whitespace stripped

## Goal

Strip trailing whitespace from every line. Runs post-encoding as double-check — encoding already `trim_end()` but later formatters may reintroduce whitespace.

## Reference

- `cclinter/src/formatter/encoding.rs` — `trim_end()` covers this
- Design doc: "must strip trailing whitespace"

## Implementation

### 1. formatter/trailing_ws.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_trailing_ws(
    source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if source.content.is_empty() {
        return Ok(());
    }
    let had_newline = source.content.ends_with('\n');
    let lines: Vec<String> = source
        .content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}
```

## Tests

```rust
#[test]
fn strips_spaces() {
    let mut src = SourceFile::from_string("x = 1   \n", PathBuf::from("test.py"));
    fix_trailing_ws(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\n");
}

#[test]
fn strips_tabs() {
    let mut src = SourceFile::from_string("x = 1\t\n", PathBuf::from("test.py"));
    fix_trailing_ws(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\n");
}

#[test]
fn preserves_code_content() {
    let mut src = SourceFile::from_string("x = 1\ny = 2\n", PathBuf::from("test.py"));
    fix_trailing_ws(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\ny = 2\n");
}

#[test]
fn handles_empty_lines() {
    let mut src = SourceFile::from_string("x = 1\n   \ny = 2\n", PathBuf::from("test.py"));
    fix_trailing_ws(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\n\ny = 2\n");
}
```

## Verify

```bash
cargo test -- trailing_ws
```
