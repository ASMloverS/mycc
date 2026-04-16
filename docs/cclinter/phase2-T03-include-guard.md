### Task 03: Include Guard + Duplicate Include Detection

**Files:**
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Create: `tools/linter/cclinter/src/checker/include_guard.rs`
- Test: `tests/checker_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_missing_include test_has_pragma test_duplicate_include test_no_guard`
Expected: FAIL.

- [ ] **Step 3: Create `src/checker/include_guard.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::HashSet;

pub fn check_includes(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let include_re = Regex::new(r#"#\s*include\s+[<"]([^>"]+)[>"]"#).unwrap();
    let is_header = source
        .path
        .extension()
        .map(|e| e == "h")
        .unwrap_or(false);

    let mut seen = HashSet::new();
    for (line_num, line) in source.lines.iter().enumerate() {
        if let Some(caps) = include_re.captures(line) {
            let header = &caps[1];
            if seen.contains(header) {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    line_num + 1,
                    1,
                    Severity::Warning,
                    "bugprone-duplicate-include",
                    &format!("Duplicate include: {}", header),
                    line,
                ));
            }
            seen.insert(header.to_string());
        }
    }

    if is_header {
        let has_guard = source.lines.iter().any(|l| l.trim().starts_with("#pragma once"))
            || source.lines.iter().any(|l| l.contains("#ifndef") && l.contains("_H"));
        if !has_guard && !source.lines.is_empty() {
            diags.push(Diagnostic::new(
                source.path.to_string_lossy().to_string(),
                1,
                1,
                Severity::Warning,
                "bugprone-missing-include-guard",
                "Header file is missing an include guard",
            ));
        }
    }

    diags
}
```

- [ ] **Step 4: Register module, update `check_source`**

Add `pub mod include_guard;` to `src/checker/mod.rs`.

- [ ] **Step 5: Run tests**

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): include guard and duplicate include detection"
```
