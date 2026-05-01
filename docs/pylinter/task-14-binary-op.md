# Task 14: Formatter — binary_op Line Break

> Status: ⬜ Not started
> Depends: Task 05
> Output: Long expression binary operators moved to line start or end per config

## Goal

Binary expression line breaks:
- `before` (default): operator at line start (PEP 8)
- `after`: operator at line end

## PEP 8 Rule

PEP 8 recommends operator at line start after break (W504 style):

```python
# Before (operator line start - PEP 8):
income = (gross_wages
          + taxable_interest
          + (dividends - qualified_dividends)
          - ira_deduction
          - student_loan_interest)
```

## Impl Steps

### 1. formatter/binary_op.rs

```rust
use crate::common::source::SourceFile;
use crate::config::{BinaryOpBreak, FormatConfig};

pub fn fix_binary_op(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if source.content.is_empty() {
        return Ok(());
    }

    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();
    let mut result: Vec<String> = Vec::with_capacity(lines.len());

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if needs_binary_op_fix(line) {
            let fixed = fix_line_binary_op(line, config);
            result.extend(fixed);
        } else {
            result.push(line.to_string());
        }
        i += 1;
    }

    let joined = result.join("\n");
    source.content = if had_newline && !joined.is_empty() {
        format!("{}\n", joined)
    } else {
        joined
    };
    Ok(())
}

fn needs_binary_op_fix(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with('+')
        || trimmed.ends_with('-')
        || trimmed.ends_with('*')
        || trimmed.ends_with('/')
        || trimmed.ends_with("and")
        || trimmed.ends_with("or")
}

fn fix_line_binary_op(line: &str, config: &FormatConfig) -> Vec<String> {
    match config.binary_op_line_break {
        BinaryOpBreak::Before => {
            move_op_to_next_line(line)
        }
        BinaryOpBreak::After => {
            vec![line.to_string()]
        }
    }
}

fn move_op_to_next_line(line: &str) -> Vec<String> {
    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    // ... impl
}
```

## Tests

```rust
#[test]
fn move_op_to_next_line_before_style() {
    let input = "x = a +\n    b\n";
    let expected = "x = a\n    + b\n";
    let config = FormatConfig { binary_op_line_break: BinaryOpBreak::Before, ..Default::default() };
}

#[test]
fn keep_op_at_line_end_after_style() {
    let input = "x = a +\n    b\n";
    // should not change
    let config = FormatConfig { binary_op_line_break: BinaryOpBreak::After, ..Default::default() };
}

#[test]
fn and_or_operators() {
    let input = "if a and\n    b:\n    pass\n";
    let expected = "if a\n    and b:\n    pass\n";
}

#[test]
fn no_change_if_not_continuation() {
    let input = "x = 1 + 2\n";
    // should not change
}
```

## Verify

```bash
cargo test -- binary_op
```
