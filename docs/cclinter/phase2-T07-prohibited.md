### Task 07: Prohibited Function Check + YAML Extend/Remove

**Files:**
- Create: `tools/linter/cclinter/src/checker/prohibited.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_default_prohibited test_extra_prohibited test_remove_from_default`
Expected: FAIL.

- [ ] **Step 3: Create `src/checker/prohibited.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::HashSet;

const DEFAULT_PROHIBITED: &[&str] = &["strcpy", "strcat", "sprintf", "vsprintf", "gets", "scanf"];

pub fn check_prohibited(source: &SourceFile, use_default: bool, extra: &[String], remove: &[String]) -> Vec<Diagnostic> {
    let mut fns: HashSet<String> = HashSet::new();
    if use_default {
        fns.extend(DEFAULT_PROHIBITED.iter().map(|s| s.to_string()));
    }
    for e in extra {
        fns.insert(e.clone());
    }
    for r in remove {
        fns.remove(r);
    }

    let mut diags = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if line.trim().starts_with('#') || line.trim().starts_with("//") {
            continue;
        }
        for fn_name in &fns {
            let pattern = format!(r"\b{}\s*\(", regex::escape(fn_name));
            if let Ok(re) = Regex::new(&pattern) {
                if re.is_match(line) {
                    diags.push(Diagnostic::new_with_source(
                        source.path.to_string_lossy().to_string(),
                        i + 1,
                        1,
                        Severity::Error,
                        "bugprone-prohibited-function",
                        &format!("Use of prohibited function: {}", fn_name),
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

Add `pub mod prohibited;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): prohibited function check with YAML extend/remove"
```
