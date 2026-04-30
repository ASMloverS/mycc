### Task 03: Include Guard + Duplicate Include Detection

**Files:**
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Create: `tools/linter/cclinter/src/checker/include_guard.rs`
- Test: `tests/checker_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/checker_tests.rs`:

```rust
use cclinter::checker::include_guard::check_includes;

#[test]
fn test_missing_include_guard() {
    let input = "int x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let diags = check_includes(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_has_pragma_once() {
    let input = "#pragma once\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let diags = check_includes(&src);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_duplicate_include() {
    let input = "#include <stdio.h>\n#include <stdlib.h>\n#include <stdio.h>\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_includes(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-duplicate-include"));
}

#[test]
fn test_no_guard_for_c_file() {
    let input = "int x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_includes(&src);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_missing_include test_has_pragma test_duplicate_include test_no_guard`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/include_guard.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::{IncludeGuardConfig, IncludeGuardStyle};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

pub fn check_include_guard(source: &SourceFile, config: &IncludeGuardConfig) -> Vec<Diagnostic> {
    // 1. Duplicate include detection using HashSet<String>
    // 2. Header guard check for .h, .hpp, .hh, .hxx files
    //    - PragmaOnce style: checks for `#pragma once`
    //    - Ifndef style: checks first 10 lines for `#ifndef`
    //    - Config style determines which to check
}
```

Key: takes `&IncludeGuardConfig` parameter. Header extensions: `h`, `hpp`, `hh`, `hxx`. Guard check respects `config.style` (PragmaOnce or Ifndef). Rule IDs: `bugprone-duplicate-include`, `bugprone-missing-include-guard`.

- [x] **Step 4: Register module, update `check_source`**

Add `pub mod include_guard;` to `src/checker/mod.rs`.

- [x] **Step 5: Run tests**

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "鉁?feat(cclinter): include guard and duplicate include detection"
```
