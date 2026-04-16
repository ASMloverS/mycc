### Task 08: Pointer Alignment — `int *p` → `int* p`

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/pointer_style.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::pointer_style::fix_pointer_style;

#[test]
fn test_pointer_left_align() {
    let input = "int *p;\nchar *s;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_pointer_style(&src, "left");
    assert!(result.content.contains("int* p"));
    assert!(result.content.contains("char* s"));
}

#[test]
fn test_pointer_right_align() {
    let input = "int* p;\nchar* s;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_pointer_style(&src, "right");
    assert!(result.content.contains("int *p"));
    assert!(result.content.contains("char *s"));
}

#[test]
fn test_pointer_no_change_when_correct() {
    let input = "int* p;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_pointer_style(&src, "left");
    assert_eq!(result.content, input);
}

#[test]
fn test_double_pointer() {
    let input = "int **pp;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_pointer_style(&src, "left");
    assert!(result.content.contains("int** pp"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_pointer`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/pointer_style.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_pointer_style(source: &SourceFile, alignment: &str) -> SourceFile {
    let lines: Vec<String> = source
        .lines
        .iter()
        .map(|line| fix_pointer_line(line, alignment))
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

fn fix_pointer_line(line: &str, alignment: &str) -> String {
    if line.trim_start().starts_with('#') || line.trim_start().starts_with("//") {
        return line.to_string();
    }
    match alignment {
        "left" => {
            let re = Regex::new(r"\b(\w+)\s+(\*+)\s+(\w+)").unwrap();
            re.replace_all(line, |caps: &regex::Captures| {
                format!("{}{} {}", &caps[1], &caps[2], &caps[3])
            })
            .to_string()
        }
        "right" => {
            let re = Regex::new(r"\b(\w+)(\*+)\s+(\w+)").unwrap();
            re.replace_all(line, |caps: &regex::Captures| {
                format!("{} {}{}", &caps[1], &caps[2], &caps[3])
            })
            .to_string()
        }
        _ => line.to_string(),
    }
}
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod pointer_style;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = pointer_style::fix_pointer_style(
    &source,
    config.format.pointer_alignment.as_deref().unwrap_or("left"),
);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): pointer alignment style (left/right)"
```
