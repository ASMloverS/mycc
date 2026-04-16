### Task 03: strict Level — Suspicious Casts, Dead Branches, Resource Leaks

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/strict.rs`
- Test: `tests/analyzer_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests test_malloc_no_free test_if_false test_suspicious_cast`
Expected: FAIL.

- [ ] **Step 3: Implement `src/analyzer/strict.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;

pub fn check(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_resource_leaks(source));
    diags.extend(check_dead_branches(source));
    diags.extend(check_suspicious_casts(source));
    diags
}

fn check_resource_leaks(source: &SourceFile) -> Vec<Diagnostic> {
    let alloc_re = Regex::new(r"\b(malloc|calloc|realloc)\s*\(").unwrap();
    let free_re = Regex::new(r"\bfree\s*\(").unwrap();
    let mut alloc_count = 0usize;
    let mut free_count = 0usize;
    let mut alloc_lines: Vec<(usize, String)> = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if alloc_re.is_match(line) {
            alloc_count += 1;
            alloc_lines.push((i + 1, line.clone()));
        }
        if free_re.is_match(line) {
            free_count += 1;
        }
    }
    let mut diags = Vec::new();
    if alloc_count > free_count {
        for (line_num, line) in &alloc_lines {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                *line_num, 1,
                Severity::Warning,
                "bugprone-resource-leak",
                "Allocated memory may not be freed",
                line,
            ));
        }
    }
    diags
}

fn check_dead_branches(source: &SourceFile) -> Vec<Diagnostic> {
    let re = Regex::new(r"^\s*if\s*\(\s*(0|false)\s*\)").unwrap();
    let mut diags = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if re.is_match(line) {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                i + 1, 1,
                Severity::Warning,
                "bugprone-dead-branch",
                "Condition is always false",
                line,
            ));
        }
    }
    diags
}

fn check_suspicious_casts(source: &SourceFile) -> Vec<Diagnostic> {
    let re = Regex::new(r"\(\s*int\s*\)\s*\w+\s*;").unwrap();
    let ptr_decl_re = Regex::new(r"\b\w+\s*\*\s*\w+").unwrap();
    let mut diags = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if re.is_match(line) {
            let nearby = source.lines.iter().take(i + 5).skip(i.saturating_sub(3));
            let has_ptr = nearby.any(|l| ptr_decl_re.is_match(l) || l.contains("ptr") || l.contains("pointer"));
            if has_ptr || line.contains("ptr") {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    i + 1, 1,
                    Severity::Warning,
                    "bugprone-suspicious-cast",
                    "Casting pointer to int may lose data",
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
git commit -m "✨ feat(cclinter): strict analysis — resource leaks, dead branches, suspicious casts"
```
