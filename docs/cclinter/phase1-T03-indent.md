### Task 03: Indentation — Tab → 2-Space, Brace-Level Indent

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/indent.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::indent::fix_indent;

#[test]
fn test_tab_to_spaces() {
    let input = "int main() {\n\tint x = 1;\n\treturn 0;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_indent(&src, 2);
    assert_eq!(result.content, "int main() {\n  int x = 1;\n  return 0;\n}\n");
}

#[test]
fn test_nested_indent() {
    let input = "void f() {\n\tif (1) {\n\t\treturn;\n\t}\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_indent(&src, 2);
    assert_eq!(result.content, "void f() {\n  if (1) {\n    return;\n  }\n}\n");
}

#[test]
fn test_indent_preserves_leading_spaces() {
    let input = "int x;\n    int y;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_indent(&src, 2);
    assert_eq!(result.content, "int x;\n    int y;\n");
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_tab_to_spaces test_nested_indent test_indent_preserves_leading_spaces`
Expected: FAIL — `fix_indent` does not exist.

- [x] **Step 3: Create `src/formatter/indent.rs`**

```rust
use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;

pub fn fix_indent(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.use_tabs { return Ok(()); }
    let indent_width = config.indent_width;
    // ...
    // Tracks brace depth and paren depth for indentation.
    // Uses split_outside_strings to avoid counting braces inside string literals.
    // Handles block comments, preprocessor directives (skip).
    // Extra indent when inside paren expressions.
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`, not individual params.

- [x] **Step 4: Register module and update pipeline**

Add `pub mod indent;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
indent::fix_indent(source, config)?;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): tab-to-space indentation with configurable width"
```
