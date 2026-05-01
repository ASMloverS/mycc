# Task 21: Checker Pipeline Integration + Phase 2 Formatter Completion

> Status: ⬜ Not started
> Depends on: Task 10–20
> Output: Complete format + check pipeline, checker wired in cli.rs

## Goals

1. Wire Phase 2 formatter modules (import_sort, comment_style, line_length, binary_op) into pipeline
2. Implement checker pipeline (naming, complexity, magic_number, unused_import, prohibited, docstring)
3. Wire checker in cli.rs, exit code logic matching cclinter

## Reference

- `cclinter/src/formatter/mod.rs` — pipeline wiring
- `cclinter/src/checker/mod.rs` — checker wiring
- `cclinter/src/cli.rs` — checker invocation logic

## Steps

### 1. formatter/mod.rs — full pipeline

```rust
pub fn format_source(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
    encoding::fix_encoding(source, config)?;
    trailing_ws::fix_trailing_ws(source, config)?;
    indent::fix_indent(source, config)?;
    blank_lines::fix_blank_lines(source, config)?;
    import_sort::fix_import_sort(source, config)?;
    comment_style::fix_comment_style(source, config)?;
    line_length::fix_line_length(source, config)?;
    binary_op::fix_binary_op(source, config)?;
    Ok(vec![])
}
```

### 2. checker/mod.rs

```rust
pub mod complexity;
pub mod docstring;
pub mod magic_number;
pub mod naming;
pub mod prohibited;
pub mod unused_import;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::CheckConfig;

pub fn check_source(source: &SourceFile, config: &CheckConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(naming::check_naming(source, &config.naming));
    diags.extend(complexity::check_complexity(source, &config.complexity));
    diags.extend(magic_number::check_magic_number(source, &config.magic_number));
    diags.extend(unused_import::check_unused_import(source, &config.unused_import));
    diags.extend(prohibited::check_prohibited(source, &config.prohibited));
    diags.extend(docstring::check_docstring(source, &config.docstring));
    diags
}
```

### 3. cli.rs — checker wiring

Mirror cclinter cli.rs check logic:

```rust
if !args.format_only {
    let check_config = &config.check;
    let all_diags: Vec<Diagnostic> = files
        .par_iter()
        .flat_map(|file_path| {
            let source = SourceFile::load(file_path).unwrap_or_else(...);
            checker::check_source(&source, check_config)
        })
        .collect();
    // Output diags, exit_code |= 2
}
```

### 4. End-to-end tests

**tests/fixtures/input/full_test.py**:
```python
import sys
import os
import requests
x=1
MyConstant =42
def BadName():
    eval("1+1")
    pass
```

**tests/checker_tests.rs**:
```rust
#[test]
fn check_full_test() {
    // Verify all checkers run, produce correct diagnostic count/type
}
```

## Verify

```bash
cargo test -- checker
cargo test -- formatter
cargo run -- --check tests/fixtures/input/full_test.py
```
