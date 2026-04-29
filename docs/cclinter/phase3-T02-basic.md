### Task 02: basic Level — Implicit Conv, Missing Return, Uninit Hints

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/basic.rs`
- Test: `tests/analyzer_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/analyzer_tests.rs`:

```rust
use cclinter::analyzer::basic;

#[test]
fn test_non_void_missing_return() {
    let input = "int foo() { int x = 1; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_implicit_int_conversion() {
    let input = "float x = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_void_no_return_ok() {
    let input = "void foo() { return; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests test_non_void test_implicit test_void`
Expected: FAIL.

- [x] **Step 3: Implement `src/analyzer/basic.rs`**

Three rules implemented:
- `bugprone-missing-return` — detects non-void functions without return
- `bugprone-implicit-conversion` — detects integer-to-float/double assignment
- `bugprone-uninit` — detects potentially uninitialized local variables

Helper functions:
- `strip_line_comment` — masks `//` comments before regex matching
- `line_has_return` — unified return detection using `contains`

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;
```

```rust
pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_missing_return(source));
    diags.extend(check_implicit_conversion(source));
    diags.extend(check_uninit_hints(source));
    diags
}
```

- [x] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): basic analysis — missing return, implicit conversion"
```
