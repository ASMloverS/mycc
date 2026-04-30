### Task 08: Forward Declaration Check

**Files:**
- Create: `tools/linter/cclinter/src/checker/forward_decl.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [x] **Step 1: Write failing tests**

```rust
use cclinter::checker::forward_decl::check_forward_decls;

#[test]
fn test_missing_forward_decl() {
    let input = "void foo() { bar(); }\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decls(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-forward-declaration" && d.message.contains("bar")));
}

#[test]
fn test_has_forward_decl() {
    let input = "void bar();\nvoid foo() { bar(); }\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decls(&src);
    assert!(!diags.iter().any(|d| d.message.contains("bar")));
}

#[test]
fn test_no_call_no_issue() {
    let input = "void foo() {}\nvoid bar() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_forward_decls(&src);
    assert!(diags.is_empty());
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_missing_forward test_has_forward test_no_call_no_issue`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/forward_decl.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

// C_KEYWORDS set: if, else, for, while, do, switch, case, default, break, continue, return, goto, sizeof, typeof, alignof, offsetof, struct, union, enum

pub fn check_forward_decl(source: &SourceFile) -> Vec<Diagnostic> {
    // 1. Mask strings/comments in all lines
    // 2. Collect forward_decls: HashMap<String, usize> (name → first line)
    // 3. Collect func_defs: HashMap<String, usize> (name → first line)
    //    - Handles pending signatures (fn sig on one line, { on next)
    // 4. Second pass: for each call site, check if function is defined
    //    after the call (def_line > i) with no forward declaration
    // 5. Skip C keywords, skip already-reported names
}
```

Key: takes only `&SourceFile` (no config). Uses `HashMap` for decl/def positions (tracks line numbers). Handles pending signatures. Two-pass approach (collect → check). Filters C keywords via `HashSet`. Rule ID: `bugprone-missing-forward-declaration`.

- [x] **Step 4: Register module, run tests**

Add `pub mod forward_decl;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): forward declaration check"
```
