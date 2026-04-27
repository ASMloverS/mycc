### Task 06: Unused Variable/Macro/Parameter Detection

**Files:**
- Create: `tools/linter/cclinter/src/checker/unused.rs`
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Test: `tests/checker_tests.rs`

- [x] **Step 1: Write failing tests**

```rust
use cclinter::checker::unused::check_unused;

#[test]
fn test_unused_variable() {
    let input = "void f() {\n  int x = 1;\n  int y = x + 1;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("y")));
}

#[test]
fn test_unused_macro() {
    let input = "#define UNUSED_MACRO 42\nint main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-macro"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests test_unused_variable test_unused_macro`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/unused.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub fn check_unused(source: &SourceFile) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_unused_vars(source));
    diags.extend(check_unused_macros(source));
    diags
}

fn check_unused_vars(source: &SourceFile) -> Vec<Diagnostic> {
    let decl_re = Regex::new(r"\b(?:int|char|float|double|long|short|void|unsigned|static|\w+_t)\s+\*?\s*(\w+)\s*[=;]").unwrap();
    let mut declared: HashMap<String, (usize, String)> = HashMap::new();
    let mut used: HashSet<String> = HashSet::new();
    let use_re = Regex::new(r"\b(\w+)\b").unwrap();
    for (i, line) in source.lines.iter().enumerate() {
        for caps in decl_re.captures_iter(line) {
            let name = caps[1].to_string();
            declared.entry(name).or_insert((i + 1, line.clone()));
        }
        for caps in use_re.captures_iter(line) {
            used.insert(caps[1].to_string());
        }
    }
    let mut diags = Vec::new();
    for (name, (line_num, line)) in &declared {
        let count = used.iter().filter(|u| *u == name).count();
        if count <= 1 {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                *line_num, 1,
                Severity::Warning,
                "bugprone-unused-variable",
                &format!("Variable '{}' is unused", name),
                line,
            ));
        }
    }
    diags
}

fn check_unused_macros(source: &SourceFile) -> Vec<Diagnostic> {
    let define_re = Regex::new(r"#define\s+(\w+)").unwrap();
    let mut defined: HashMap<String, (usize, String)> = HashMap::new();
    let use_re = Regex::new(r"\b(\w+)\b").unwrap();
    for (i, line) in source.lines.iter().enumerate() {
        if let Some(caps) = define_re.captures(line) {
            let name = caps[1].to_string();
            defined.entry(name).or_insert((i + 1, line.clone()));
        }
    }
    let full_text = source.content.replace('#', " ");
    let mut diags = Vec::new();
    for (name, (line_num, line)) in &defined {
        let full_re = Regex::new(&format!(r"\b{}\b", regex::escape(name))).unwrap();
        let count = full_re.find_iter(&full_text).count();
        if count <= 1 {
            diags.push(Diagnostic::new_with_source(
                source.path.to_string_lossy().to_string(),
                *line_num, 1,
                Severity::Warning,
                "bugprone-unused-macro",
                &format!("Macro '{}' is unused", name),
                line,
            ));
        }
    }
    diags
}
```

- [x] **Step 4: Register module, run tests**

Add `pub mod unused;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): unused variable and macro detection"
```
