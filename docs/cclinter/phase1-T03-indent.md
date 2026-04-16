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

- [ ] **Step 3: Create `src/formatter/indent.rs`**

```rust
use crate::common::source::SourceFile;
use std::path::PathBuf;

pub fn fix_indent(source: &SourceFile, indent_width: usize) -> SourceFile {
    let lines: Vec<String> = source
        .lines
        .iter()
        .map(|line| {
            let leading_tabs: usize = line.chars().take_while(|&c| c == '\t').count();
            if leading_tabs == 0 {
                return line.clone();
            }
            let rest = &line[leading_tabs..];
            let spaces = " ".repeat(leading_tabs * indent_width);
            format!("{}{}", spaces, rest)
        })
        .collect();
    let content = lines.join("\n");
    let has_newline = source.content.ends_with('\n');
    let final_content = if has_newline && !content.is_empty() {
        format!("{}\n", content)
    } else {
        content
    };
    SourceFile::from_string(&final_content, source.path.clone())
}
```

- [ ] **Step 4: Register module and update pipeline**

Add `pub mod indent;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
pub fn format_source(source: &SourceFile, config: &crate::config::Config) -> SourceFile {
    let source = encoding::fix_encoding(source);
    let indent = config.format.indent_width.unwrap_or(2);
    let source = indent::fix_indent(&source, indent);
    source
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): tab-to-space indentation with configurable width"
```
