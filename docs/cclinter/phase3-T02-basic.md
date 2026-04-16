### Task 02: basic Level — Implicit Conv, Missing Return, Uninit Hints

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/basic.rs`
- Test: `tests/analyzer_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests test_non_void test_implicit test_void`
Expected: FAIL.

- [ ] **Step 3: Implement `src/analyzer/basic.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;

pub fn check(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_missing_return(source));
    diags.extend(check_implicit_conversion(source));
    diags
}

fn check_missing_return(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let fn_re = Regex::new(r"^\s*(int|char|float|double|long|short|unsigned|\w+_t)\s+\*?\s*(\w+)\s*\([^)]*\)\s*\{").unwrap();
    let mut in_fn = false;
    let mut fn_line = 0;
    let mut brace_depth = 0i32;
    let mut has_return = false;
    let mut return_type = String::new();

    for (i, line) in source.lines.iter().enumerate() {
        let trimmed = line.trim();
        if !in_fn {
            if let Some(caps) = fn_re.captures(line) {
                return_type = caps[1].to_string();
                if return_type != "void" {
                    in_fn = true;
                    fn_line = i + 1;
                    has_return = false;
                    brace_depth = trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
                }
            }
        } else {
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;
            if trimmed.starts_with("return") {
                has_return = true;
            }
            if brace_depth <= 0 {
                if !has_return {
                    diags.push(Diagnostic::new(
                        source.path.to_string_lossy().to_string(),
                        fn_line, 1,
                        Severity::Warning,
                        "bugprone-missing-return",
                        &format!("Non-void function may missing return statement"),
                    ));
                }
                in_fn = false;
            }
        }
    }
    diags
}

fn check_implicit_conversion(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let conv_re = Regex::new(r"\b(float|double)\s+(\w+)\s*=\s*(\d+)\s*;").unwrap();
    for (i, line) in source.lines.iter().enumerate() {
        if let Some(caps) = conv_re.captures(line) {
            let val = &caps[3];
            if !val.contains('.') {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    i + 1,
                    1,
                    Severity::Warning,
                    "bugprone-implicit-conversion",
                    &format!("Implicit integer to {} conversion", &caps[1]),
                    line,
                ));
            }
        }
    }
    diags
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): basic analysis — missing return, implicit conversion"
```
