### Task 04: Complexity — Function/File Line Count, Nesting Depth

**Files:**
- Create: `tools/linter/cclinter/src/checker/complexity.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [ ] **Step 1: Write failing tests**

```rust
use cclinter::checker::complexity::check_complexity;

#[test]
fn test_function_too_long() {
    let lines: Vec<String> = (0..110).map(|i| format!("  int x{} = {};", i, i)).collect();
    let input = format!("void long_fn() {{\n{}\n}}\n", lines.join("\n"));
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, 100, 2000, 5);
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_file_too_long() {
    let lines: Vec<String> = (0..2100).map(|i| format!("int x{} = {};", i, i)).collect();
    let input = lines.join("\n") + "\n";
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, 100, 2000, 5);
    assert!(diags.iter().any(|d| d.rule_id == "readability-file-size"));
}

#[test]
fn test_deep_nesting() {
    let input = "void f() {\n  if (1) {\n    if (2) {\n      if (3) {\n        if (4) {\n          if (5) {\n            if (6) {\n            }\n          }\n        }\n      }\n    }\n  }\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, 100, 2000, 5);
    assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_function_too_long test_file_too_long test_deep_nesting`
Expected: FAIL.

- [ ] **Step 3: Create `src/checker/complexity.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;

pub fn check_complexity(source: &SourceFile, max_fn_lines: usize, max_file_lines: usize, max_nesting: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if source.lines.len() > max_file_lines {
        diags.push(Diagnostic::new(
            source.path.to_string_lossy().to_string(),
            1, 1,
            Severity::Warning,
            "readability-file-size",
            &format!("File has {} lines (max {})", source.lines.len(), max_file_lines),
        ));
    }
    diags.extend(check_function_lengths(source, max_fn_lines));
    diags.extend(check_nesting_depth(source, max_nesting));
    diags
}

fn check_function_lengths(source: &SourceFile, max_lines: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut fn_start = None;
    let mut brace_depth = 0i32;
    let fn_re = regex::Regex::new(r"^\s*\w+\s+\*?\s*\w+\s*\(").unwrap();
    for (i, line) in source.lines.iter().enumerate() {
        let trimmed = line.trim();
        if fn_start.is_none() && fn_re.is_match(trimmed) && trimmed.contains('{') {
            fn_start = Some(i);
            brace_depth = trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
            continue;
        }
        if let Some(start) = fn_start {
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;
            if brace_depth <= 0 {
                let len = i - start;
                if len > max_lines {
                    diags.push(Diagnostic::new(
                        source.path.to_string_lossy().to_string(),
                        start + 1, 1,
                        Severity::Warning,
                        "readability-function-size",
                        &format!("Function spans {} lines (max {})", len, max_lines),
                    ));
                }
                fn_start = None;
            }
        }
    }
    diags
}

fn check_nesting_depth(source: &SourceFile, max_depth: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let mut depth = 0usize;
    for (i, line) in source.lines.iter().enumerate() {
        depth += line.matches('{').count();
        if depth > max_depth {
            diags.push(Diagnostic::new(
                source.path.to_string_lossy().to_string(),
                i + 1, 1,
                Severity::Warning,
                "readability-deep-nesting",
                &format!("Nesting depth {} exceeds max {}", depth, max_depth),
            ));
        }
        depth = depth.saturating_sub(line.matches('}').count());
    }
    diags
}
```

- [ ] **Step 4: Register module**

Add `pub mod complexity;` to `src/checker/mod.rs`.

- [ ] **Step 5: Run tests**

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): complexity checker (fn/file size, nesting depth)"
```
