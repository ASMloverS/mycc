### Task 05: Brace Style — Google Attach

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/braces.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::braces::fix_braces;

#[test]
fn test_breakout_to_attach_function() {
    let input = "void f()\n{\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_braces(&src, "attach");
    assert!(result.content.starts_with("void f() {"));
}

#[test]
fn test_breakout_to_attach_if() {
    let input = "if (x)\n{\n  y();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_braces(&src, "attach");
    assert!(result.content.starts_with("if (x) {"));
}

#[test]
fn test_breakout_to_attach_else() {
    let input = "if (x) {\n} else\n{\n  y();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_braces(&src, "attach");
    assert!(result.content.contains("} else {"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_breakout_to_attach`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/braces.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_braces(source: &SourceFile, brace_style: &str) -> SourceFile {
    if brace_style != "attach" {
        return source.clone();
    }
    let re = Regex::new(r"(?P<keyword>[^/\n]+?)\s*\n\s*\{").unwrap();
    let mut content = source.content.clone();
    content = re.replace_all(&content, "$keyword {").to_string();
    SourceFile::from_string(&content, source.path.clone())
}
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod braces;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = braces::fix_braces(&source, config.format.brace_style.as_deref().unwrap_or("attach"));
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): Google attach brace style"
```
