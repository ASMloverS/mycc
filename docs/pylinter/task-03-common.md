# Task 03: Common Module — SourceFile + Diagnostic + string_utils

> Status: ⬜ Not started
> Deps: Task 01
> Output: `common` module importable, unit tests pass

## Goal

Build `common` module: SourceFile, Diagnostic, string_utils. Isomorphic to cclinter but standalone.

## Reference

- `cclinter/src/common/source.rs` — SourceFile
- `cclinter/src/common/diag.rs` — Diagnostic + Severity
- `cclinter/src/common/string_utils.rs` — split_outside_strings

## Steps

### 1. common/mod.rs

```rust
pub mod diag;
pub mod source;
pub mod string_utils;
```

### 2. common/source.rs

Copy from cclinter, adapt:

```rust
#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub original: String,
}

impl SourceFile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> { ... }
    pub fn from_string(content: &str, path: PathBuf) -> Self { ... }
    pub fn lines(&self) -> Vec<&str> { ... }
    pub fn line_count(&self) -> usize { ... }
    pub fn is_modified(&self) -> bool { ... }
    pub fn display_path(&self) -> String { ... }
}
```

> Skip C's `mask_string_literals`, `strip_line_comment`, `mask_code_line` — Python version handles these in CST module.

### 3. common/diag.rs

Identical to cclinter:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Severity { Note, Warning, Error }

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
    pub source_line: Option<String>,
}

// Display impl: "{file}:{line}:{col}: {severity}: {message} [{rule_id}]"
```

### 4. common/string_utils.rs

```rust
pub fn split_outside_strings(s: &str) -> Vec<String> { ... }
```

Identical to cclinter. Splits on whitespace outside `'` and `"` strings.

## Tests

```rust
// source.rs tests
#[test] fn from_string_and_line_access() { ... }
#[test] fn modification_tracking() { ... }

// diag.rs tests
#[test] fn display_format() { ... }

// string_utils.rs tests
#[test] fn split_basic() { ... }
#[test] fn split_with_strings() { ... }
```

## Verify

```bash
cargo test -- common
```
