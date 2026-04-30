### Task 04: Complexity — Function/File Line Count, Nesting Depth

**Files:**
- Create: `tools/linter/cclinter/src/checker/complexity.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [x] **Step 1: Write failing tests**

```rust
use cclinter::checker::complexity::check_complexity;

#[test]
fn test_function_too_long() {
    let lines: Vec<String> = (0..110).map(|i| format!("  int x{} = {};", i, i)).collect();
    let input = format!("void long_fn() {{\n{}\n}}\n", lines.join("\n"));
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, 100, 2000, 5);
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_file_too_long() {
    let lines: Vec<String> = (0..2100).map(|i| format!("int x{} = {};", i, i)).collect();
    let input = lines.join("\n") + "\n";
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, 100, 2000, 5);
    assert!(diags.iter().any(|d| d.rule_id == "readability-file-size"));
}

#[test]
fn test_deep_nesting() {
    let input = "void f() {\n  if (1) {\n    if (2) {\n      if (3) {\n        if (4) {\n          if (5) {\n            if (6) {\n            }\n          }\n        }\n      }\n    }\n  }\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, 100, 2000, 5);
    assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_function_too_long test_file_too_long test_deep_nesting`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/complexity.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::ComplexityConfig;
use regex::Regex;
use std::sync::LazyLock;

pub fn check_complexity(source: &SourceFile, config: &ComplexityConfig) -> Vec<Diagnostic> {
    // 1. File line count check: readability-file-size
    // 2. Function length check: readability-function-size
    //    - Tracks pending signatures (fn signature on one line, { on next)
    //    - Handles single-line functions { ... }
    // 3. Nesting depth check: readability-deep-nesting
    //    - Only emits first occurrence (was_over tracking)
}
```

Key: takes `&ComplexityConfig` (not individual params). Uses `source.lines()` method. Handles pending signatures (function header + `{` on separate line). Nesting check uses `saturating_sub` and emits only once per nesting violation region.

- [x] **Step 4: Register module**

Add `pub mod complexity;` to `src/checker/mod.rs`.

- [x] **Step 5: Run tests**

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): complexity checker (fn/file size, nesting depth)"
```
