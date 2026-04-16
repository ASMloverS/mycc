### Task 05: Magic Number Detection + Allowlist

**Files:**
- Create: `tools/linter/cclinter/src/checker/magic_number.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_detect_magic test_allowed_numbers test_disabled`
Expected: FAIL.

- [ ] **Step 3: Create `src/checker/magic_number.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::HashSet;

pub fn check_magic_numbers(source: &SourceFile, enabled: bool, allowed: &[i64]) -> Vec<Diagnostic> {
    if !enabled {
        return vec![];
    }
    let allowed_set: HashSet<i64> = allowed.iter().copied().collect();
    let num_re = Regex::new(r"(?<![a-zA-Z_0-9])(\-?\d+)(?![a-zA-Z_0-9.xX])").unwrap();
    let mut diags = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if line.trim().starts_with('#') || line.trim().starts_with("//") {
            continue;
        }
        for caps in num_re.captures_iter(line) {
            if let Ok(val) = caps[1].parse::<i64>() {
                if !allowed_set.contains(&val) {
                    diags.push(Diagnostic::new_with_source(
                        source.path.to_string_lossy().to_string(),
                        i + 1,
                        caps.get(1).unwrap().start(),
                        Severity::Warning,
                        "readability-magic-numbers",
                        &format!("Magic number: {}", val),
                        line,
                    ));
                }
            }
        }
    }
    diags
}
```

- [ ] **Step 4: Register module, run tests**

Add `pub mod magic_number;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): magic number detection with allowlist"
```
