### Task 04: deep Level — Buffer Overflow, Null Deref Patterns

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/deep.rs`
- Test: `tests/analyzer_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/analyzer_tests.rs`:

```rust
use cclinter::analyzer::deep;

#[test]
fn test_gets_buffer_overflow() {
    let input = "char buf[10];\ngets(buf);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = deep::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-buffer-overflow-risk"));
}

#[test]
fn test_null_deref_pattern() {
    let input = "int* p = NULL;\n*p = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = deep::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-null-deref-risk"));
}

#[test]
fn test_array_no_bounds_check() {
    let input = "int arr[10];\nint idx = get_index();\narr[idx] = 1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = deep::check(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-buffer-overflow-risk"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests test_gets_buffer test_null_deref test_array_no_bounds`
Expected: FAIL.

- [x] **Step 3: Implement `src/analyzer/deep.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;

pub fn check(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_buffer_overflow_patterns(source));
    diags.extend(check_null_deref_patterns(source));
    diags
}

fn check_buffer_overflow_patterns(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let arr_decl_re = Regex::new(r"\b(\w+)\s*\[\s*(\d+)\s*\]").unwrap();
    let gets_re = Regex::new(r"\bgets\s*\(").unwrap();
    let scanf_s_re = Regex::new(r#"scanf\s*\(\s*"%s"\s*,\s*\w+\s*\)"#).unwrap();

    let mut arrays: Vec<(String, usize)> = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if let Some(caps) = arr_decl_re.captures(line) {
            let name = caps[1].to_string();
            let size: usize = caps[2].parse().unwrap_or(0);
            arrays.push((name, size));
        }
        if gets_re.is_match(line) {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                i + 1, 1,
                Severity::Error,
                "bugprone-buffer-overflow-risk",
                "gets() has no bounds checking — use fgets() instead",
                line,
            ));
        }
        if scanf_s_re.is_match(line) {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                i + 1, 1,
                Severity::Warning,
                "bugprone-buffer-overflow-risk",
                "scanf(\"%s\", ...) has no bounds checking",
                line,
            ));
        }
        for (arr_name, _arr_size) in &arrays {
            let idx_re = Regex::new(&format!(r"\b{}\s*\[\s*\w+\s*\]", regex::escape(arr_name))).unwrap();
            if idx_re.is_match(line) && !line.contains("if") && !line.contains("<") && !line.contains(">") && !line.contains("sizeof") {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    i + 1, 1,
                    Severity::Warning,
                    "bugprone-buffer-overflow-risk",
                    &format!("Array '{}' access without bounds check", arr_name),
                    line,
                ));
            }
        }
    }
    diags
}

fn check_null_deref_patterns(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let null_init_re = Regex::new(r"\b(\w+)\s*\*\s*(\w+)\s*=\s*NULL\s*;").unwrap();
    let deref_re = Regex::new(r"\*\s*(\w+)\s*=").unwrap();
    let mut null_ptrs: Vec<String> = Vec::new();
    for (i, line) in source.lines.iter().enumerate() {
        if let Some(caps) = null_init_re.captures(line) {
            null_ptrs.push(caps[2].to_string());
        }
        if let Some(caps) = deref_re.captures(line) {
            let ptr_name = &caps[1];
            if null_ptrs.contains(&ptr_name.to_string()) {
                let has_null_check = source.lines[..i].iter().rev().take(5).any(|l| {
                    l.contains("if") && l.contains(ptr_name) && (l.contains("NULL") || l.contains("null"))
                });
                if !has_null_check {
                    diags.push(Diagnostic::new_with_source(
                        source.path.to_string_lossy().to_string(),
                        i + 1, 1,
                        Severity::Warning,
                        "bugprone-null-deref-risk",
                        &format!("Potential null pointer dereference: *{}", ptr_name),
                        line,
                    ));
                }
            }
        }
    }
    diags
}
```

- [x] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): deep analysis — buffer overflow and null deref pattern detection"
```
