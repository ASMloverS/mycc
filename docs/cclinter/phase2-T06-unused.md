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
use crate::config::UnusedConfig;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

pub fn check_unused(source: &SourceFile, config: &UnusedConfig) -> Vec<Diagnostic> {
    if !config.enabled { return vec![]; }
    // check_unused_vars: declares → counts usage → flags if count <= 1
    // check_unused_macros: #define names → counts usage outside define lines → flags if count == 0
}
```

Key: takes `&UnusedConfig` (configurable `enabled` flag). Variable check uses `HashMap<String, usize>` for declaration positions. Macro check excludes define lines from usage counting. Masks string literals and block comments. Known limitation: no scope awareness (may produce false positives/negatives). Rule IDs: `bugprone-unused-variable`, `bugprone-unused-macro`.

- [x] **Step 4: Register module, run tests**

Add `pub mod unused;` to `src/checker/mod.rs`.

Run: `cargo test --test checker_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): unused variable and macro detection"
```
