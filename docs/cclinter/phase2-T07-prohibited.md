### Task 07: Prohibited Function Check + YAML Extend/Remove

**Files:**
- Create: `tools/linter/cclinter/src/checker/prohibited.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [x] **Step 1: Write failing tests**

```rust
use cclinter::checker::prohibited::check_prohibited;

#[test]
fn test_default_prohibited() {
    let input = "char buf[10];\nstrcpy(buf, src);\nsprintf(buf, \"%d\", x);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &[]);
    assert!(diags.iter().any(|d| d.message.contains("strcpy")));
    assert!(diags.iter().any(|d| d.message.contains("sprintf")));
}

#[test]
fn test_extra_prohibited() {
    let input = "malloc(10);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &["malloc".into()], &[]);
    assert!(diags.iter().any(|d| d.message.contains("malloc")));
}

#[test]
fn test_remove_from_default() {
    let input = "strcpy(buf, src);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_prohibited(&src, true, &[], &["strcpy".into()]);
    assert!(!diags.iter().any(|d| d.message.contains("strcpy")));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_default_prohibited test_extra_prohibited test_remove_from_default`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/prohibited.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

const DEFAULT_PROHIBITED: &[&str] = &["strcpy", "strcat", "sprintf", "vsprintf", "gets", "scanf"];

pub fn check_prohibited(
    source: &SourceFile,
    use_default: bool,
    extra: &[String],
    remove: &[String],
) -> Vec<Diagnostic> {
    // Build effective set: (default if use_default) + extra - remove
    // Mask string literals and block comments before matching
    // Skip preprocessor and comment lines
}
```

Key: masks string literals and block comments before regex matching (avoids false positives in strings). Builds regex patterns per function name using `regex::escape`. Rule ID: `bugprone-prohibited-function`, severity: `Error`.

- [x] **Step 4: Register module, run tests**

Add `pub mod prohibited;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): prohibited function check with YAML extend/remove"
```
