### Task 04: deep Level — Buffer Overflow, Null Deref Patterns

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/deep.rs`
- Test: `tests/analyzer_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/analyzer_tests.rs`:

```rust
use cclinter::analyzer::deep;

#[test]
fn test_gets_buffer_overflow() {
    let input = "char buf[10];\ngets(buf);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = deep::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-buffer-overflow-risk"));
}

#[test]
fn test_null_deref_pattern() {
    let input = "int* p = NULL;\n*p = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = deep::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-null-deref-risk"));
}

#[test]
fn test_array_no_bounds_check() {
    let input = "int arr[10];\nint idx = get_index();\narr[idx] = 1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = deep::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-buffer-overflow-risk"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests test_gets_buffer test_null_deref test_array_no_bounds`
Expected: FAIL.

- [x] **Step 3: Implement `src/analyzer/deep.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::{mask_string_literals, strip_line_comment, SourceFile};
use crate::config::AnalysisConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let lines = source.lines();
    let mut diags = Vec::new();
    diags.extend(check_buffer_overflow_patterns(&lines, source));
    diags.extend(check_null_deref_patterns(&lines, source));
    diags
}
```

Two rule groups:

1. **`bugprone-buffer-overflow-risk`** — Three sub-checks:
   - `gets()` usage (severity: Error)
   - `scanf("%s", ...)` without bounds (severity: Warning)
   - Array access without bounds check: collects array declarations, detects `arr[var_index]` without nearby comparison or `sizeof`

2. **`bugprone-null-deref-risk`** — Detects `*ptr` dereference where `ptr` was initialized to `NULL`:
   - Tracks `NULL_PTR_INIT_RE` declarations
   - Uses `mask_string_literals` to avoid false positives
   - `is_deref_context` distinguishes `*ptr` dereference from pointer declarations
   - Looks back `NULL_CHECK_LOOKBACK=5` lines for null checks (`if` + ptr name)
   - Function-scoped: clears null ptrs at function boundaries (brace depth transitions)

Key: takes `(source, _config: &AnalysisConfig)`. Uses `mask_string_literals` from `source.rs`. Buffer overflow checks use regex-based bounds detection.

- [x] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): deep analysis — buffer overflow and null deref pattern detection"
```
