### Task 03: strict Level — Suspicious Casts, Dead Branches, Resource Leaks

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/strict.rs`
- Test: `tests/analyzer_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/analyzer_tests.rs`:

```rust
use cclinter::analyzer::strict;

#[test]
fn test_malloc_no_free() {
    let input = "void f() {\n  void* p = malloc(100);\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_if_false_dead_branch() {
    let input = "if (0) {\n  dead_code();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_suspicious_cast() {
    let input = "int x = (int)ptr;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-suspicious-cast"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests test_malloc_no_free test_if_false test_suspicious_cast`
Expected: FAIL.

- [x] **Step 3: Implement `src/analyzer/strict.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::{strip_line_comment, SourceFile};
use crate::config::AnalysisConfig;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let lines = source.lines();
    let mut diags = Vec::new();
    diags.extend(check_resource_leaks(&lines, source));
    diags.extend(check_dead_branches(&lines, source));
    diags.extend(check_suspicious_casts(&lines, source));
    diags
}
```

Three rules:

1. **`bugprone-resource-leak`** — Function-scoped tracking: counts malloc/calloc/realloc vs free per function. Tracks allocation variable names; if function returns an allocation variable, suppresses leak warning. Uses `brace_depth` for function boundary detection.

2. **`bugprone-dead-branch`** — Detects `if (0)`, `if (false)`, `else if (0/false)`, and `#if 0` preprocessor blocks.

3. **`bugprone-suspicious-cast`** — First collects pointer declarations (`PTR_DECL_RE`), then checks if `(int)var` casts involve known pointer variables.

Key: takes `(&lines, source)`. Uses `strip_line_comment`. Resource leak detection is function-scoped (not file-scoped).

- [x] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): strict analysis — resource leaks, dead branches, suspicious casts"
```
