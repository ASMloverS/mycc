# Task 06: Formatter — encoding (UTF-8 + LF + Strip BOM)

> Status: ⬜ Not started
> Depends: Task 05
> Output: encoding formatter normalizes encoding/line-endings/BOM

## Goal

Encoding formatter:
- Strip UTF-8 BOM (`\u{feff}`)
- `\r\n` → `\n`, `\r` → `\n` (force LF)
- Guarantee valid UTF-8 output

## Reference

- `cclinter/src/formatter/encoding.rs`

## Design

Encoding = format pipeline first step. Char-level replacement only (no AST/CST) → operate `SourceFile.content` string directly, skip CSTSource.

Matches cclinter `encoding.rs` exactly.

## Steps

### 1. formatter/encoding.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_encoding(
    source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = source.content.as_str();
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let content = content.replace("\r\n", "\n").replace('\r', "\n");
    let had_newline = content.ends_with('\n');
    let lines: Vec<String> = content
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

> Note: `trim_end()` handles trailing whitespace → overlaps Task 07.
> Decision: encoding does BOM + line-endings only, trailing whitespace → Task 07.
> If `trim_end` kept here, Task 07 `trailing_ws` → no-op.
> **Recommendation**: keep `trim_end` here (match cclinter), Task 07 = independent double-check.

## Tests

```rust
#[test]
fn strips_bom() {
    let mut src = SourceFile::from_string("\u{feff}x = 1\n", PathBuf::from("test.py"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\n");
}

#[test]
fn converts_crlf_to_lf() {
    let mut src = SourceFile::from_string("x = 1\r\ny = 2\r\n", PathBuf::from("test.py"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\ny = 2\n");
}

#[test]
fn converts_cr_to_lf() {
    let mut src = SourceFile::from_string("x = 1\ry = 2\r", PathBuf::from("test.py"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\ny = 2");
}

#[test]
fn preserves_trailing_newline() {
    let mut src = SourceFile::from_string("x = 1\n", PathBuf::from("test.py"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.ends_with('\n'));
}

#[test]
fn preserves_no_trailing_newline() {
    let mut src = SourceFile::from_string("x = 1", PathBuf::from("test.py"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert!(!src.content.ends_with('\n'));
}

#[test]
fn strips_trailing_whitespace() {
    let mut src = SourceFile::from_string("x = 1  \ny = 2\t\n", PathBuf::from("test.py"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "x = 1\ny = 2\n");
}
```

## Verify

```bash
cargo test -- encoding
```
