### Task 02: Naming Convention Checks

**Files:**
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Create: `tools/linter/cclinter/src/checker/naming.rs`
- Test: `tests/checker_tests.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/checker_tests.rs`:

```rust
use cclinter::checker::naming::check_naming;
use cclinter::common::source::SourceFile;
use std::path::PathBuf;

#[test]
fn test_snake_case_function() {
    let input = "void BadFunction() {}\nvoid good_function() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "snake_case", "function");
    assert!(diags.iter().any(|d| d.message.contains("BadFunction")));
}

#[test]
fn test_upper_snake_macro() {
    let input = "#define bad_macro 1\n#define GOOD_MACRO 2\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "upper_snake_case", "macro");
    assert!(diags.iter().any(|d| d.message.contains("bad_macro")));
}

#[test]
fn test_pascal_type() {
    let input = "typedef struct bad_type {} bad_type;\ntypedef struct GoodType {} GoodType;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "pascal_case", "type");
    assert!(diags.iter().any(|d| d.message.contains("bad_type")));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests`
Expected: FAIL.

- [ ] **Step 3: Create `src/checker/naming.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;

pub fn check_naming(source: &SourceFile, style: &str, kind: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let pattern_re = naming_regex(style);
    let (search_re, extractor): (Regex, Box<dyn Fn(&regex::Captures) -> String>) = match kind {
        "function" => {
            let re = Regex::new(r"^\s*(?:void|int|char|float|double|long|short|unsigned|static|extern|const|\w+_t)\s+\*?\s*(\w+)\s*\(").unwrap();
            (re, Box::new(|caps: &regex::Captures| caps[1].to_string()))
        }
        "macro" => {
            let re = Regex::new(r"#define\s+(\w+)").unwrap();
            (re, Box::new(|caps: &regex::Captures| caps[1].to_string()))
        }
        "variable" => {
            let re = Regex::new(r"\b(?:int|char|float|double|long|void|unsigned|static|\w+_t)\s+\*?\s*(\w+)\s*[;=]").unwrap();
            (re, Box::new(|caps: &regex::Captures| caps[1].to_string()))
        }
        _ => return diags,
    };

    for (line_num, line) in source.lines.iter().enumerate() {
        for caps in search_re.captures_iter(line) {
            let name = extractor(&caps);
            if !pattern_re.is_match(&name) {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    line_num + 1,
                    1,
                    Severity::Warning,
                    &format!("readability-naming-{}", kind),
                    &format!("{} '{}' does not follow {} convention", kind, name, style),
                    line,
                ));
            }
        }
    }
    diags
}

fn naming_regex(style: &str) -> Regex {
    match style {
        "snake_case" => Regex::new(r"^[a-z][a-z0-9_]*$").unwrap(),
        "upper_snake_case" => Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap(),
        "pascal_case" => Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap(),
        "camelCase" => Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap(),
        _ => Regex::new(r".*").unwrap(),
    }
}
```

- [ ] **Step 4: Register module**

Add `pub mod naming;` to `src/checker/mod.rs`.

- [ ] **Step 5: Run tests**

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): naming convention checker (snake_case, UPPER_SNAKE, PascalCase)"
```
