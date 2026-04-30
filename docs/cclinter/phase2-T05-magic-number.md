### Task 05: Magic Number Detection + Allowlist

**Files:**
- Create: `tools/linter/cclinter/src/checker/magic_number.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [x] **Step 1: Write failing tests**

```rust
use cclinter::checker::magic_number::check_magic_numbers;

#[test]
fn test_detect_magic_number() {
    let input = "int x = 42;\nint y = 0;\nint z = 100;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_numbers(&src, true, &[0, 1, -1, 2]);
    assert!(diags.iter().any(|d| d.message.contains("42")));
    assert!(diags.iter().any(|d| d.message.contains("100")));
}

#[test]
fn test_allowed_numbers_not_flagged() {
    let input = "int x = 0;\nint y = 1;\nint z = -1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_numbers(&src, true, &[0, 1, -1, 2]);
    assert!(diags.is_empty());
}

#[test]
fn test_disabled_check() {
    let input = "int x = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_magic_numbers(&src, false, &[]);
    assert!(diags.is_empty());
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_detect_magic test_allowed_numbers test_disabled`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/magic_number.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::MagicNumberConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub fn check_magic_number(
    source: &SourceFile,
    config: &MagicNumberConfig,
) -> Vec<Diagnostic> {
    if !config.enabled { return vec![]; }
    let allowed: HashSet<i64> = config.allowed.iter().copied().collect();
    // Per-line processing:
    // 1. Skip preprocessor, comments
    // 2. mask_exclusions: mask string/char literals and block comments
    // 3. Strip trailing line comments
    // 4. Detect numbers via regex, skip floats (adjacent '.')
    // 5. Handle scientific notation exponents
    // 6. Resolve negative values (leading -)
}
```

Key: takes `&MagicNumberConfig`. Sophisticated handling: masks string literals/block comments, skips floats, handles scientific notation, resolves negative values from context. Rule ID: `readability-magic-numbers`.

- [x] **Step 4: Register module, run tests**

Add `pub mod magic_number;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): magic number detection with allowlist"
```
