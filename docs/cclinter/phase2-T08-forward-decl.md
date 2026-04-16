### Task 08: Forward Declaration Check

**Files:**
- Create: `tools/linter/cclinter/src/checker/forward_decl.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_missing_forward test_has_forward test_no_call_no_issue`
Expected: FAIL.

- [ ] **Step 3: Create `src/checker/forward_decl.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::HashSet;

pub fn check_forward_decls(source: &SourceFile) -> Vec<Diagnostic> {
    let call_re = Regex::new(r"(\w+)\s*\(").unwrap();
    let fn_def_re = Regex::new(r"^\s*\w+\s+\*?\s*(\w+)\s*\([^)]*\)\s*\{?").unwrap();
    let fwd_re = Regex::new(r"^\s*\w+\s+\*?\s*(\w+)\s*\([^)]*\)\s*;").unwrap();

    let mut forward_decls: HashSet<String> = HashSet::new();
    let mut function_defs: HashSet<String> = HashSet::new();
    let mut calls: Vec<(String, usize, String)> = Vec::new();

    for (i, line) in source.lines.iter().enumerate() {
        if let Some(caps) = fwd_re.captures(line) {
            forward_decls.insert(caps[1].to_string());
        }
        if let Some(caps) = fn_def_re.captures(line) {
            function_defs.insert(caps[1].to_string());
        }
        for caps in call_re.captures_iter(line) {
            let name = caps[1].to_string();
            if !is_keyword(&name) {
                calls.push((name, i + 1, line.clone()));
            }
        }
    }

    let mut diags = Vec::new();
    for (name, line_num, line) in &calls {
        if forward_decls.contains(name) || function_defs.contains(name) {
            continue;
        }
        let defined_after = function_defs.contains(name.as_str());
        if defined_after {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                *line_num,
                1,
                Severity::Warning,
                "bugprone-missing-forward-declaration",
                &format!("Function '{}' called before declaration", name),
                line,
            ));
            forward_decls.insert(name.clone());
        }
    }
    diags
}

fn is_keyword(name: &str) -> bool {
    matches!(name, "if" | "for" | "while" | "switch" | "return" | "sizeof" | "typeof" | "printf" | "scanf" | "malloc" | "free" | "calloc" | "realloc")
}
```

- [ ] **Step 4: Register module, run tests**

Add `pub mod forward_decl;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): forward declaration check"
```
