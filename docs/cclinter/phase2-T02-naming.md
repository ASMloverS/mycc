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

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test checker_tests`
Expected: FAIL.

- [x] **Step 3: Create `src/checker/naming.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::sync::LazyLock;

static SNAKE_CASE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap());
static UPPER_SNAKE_CASE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap());
static PASCAL_CASE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap());
static CAMEL_CASE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap());

// Separate regexes per kind: FUNCTION_RE, MACRO_RE, VARIABLE_RE, TYPE_RE, CONSTANT_RE

pub fn check_naming(source: &SourceFile, style: &str, kind: &str) -> Vec<Diagnostic> {
    let pattern_re = naming_regex(style);
    let search_re: &LazyLock<Regex> = match kind {
        "function" => &FUNCTION_RE,
        "macro" => &MACRO_RE,
        "variable" => &VARIABLE_RE,
        "type" => &TYPE_RE,
        "constant" => &CONSTANT_RE,
        _ => return vec![],
    };
    for (line_num, line) in source.lines().iter().enumerate() { ... }
}

fn naming_regex(style: &str) -> &'static LazyLock<Regex> { ... }
```

Key: uses `source.lines()` method (returns `Vec<&str>`), not `source.lines` field. Regexes are `LazyLock<Regex>` statics. Supports 5 kinds: function, macro, variable, type, constant. Rule IDs: `readability-naming-{kind}`.

- [x] **Step 4: Register module**

Add `pub mod naming;` to `src/checker/mod.rs`.

- [x] **Step 5: Run tests**

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): naming convention checker (snake_case, UPPER_SNAKE, PascalCase)"
```
