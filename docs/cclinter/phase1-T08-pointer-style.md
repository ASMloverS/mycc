### Task 08: Pointer Alignment — `int *p` → `int* p`

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/pointer_style.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

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

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_pointer`
Expected: FAIL.

- [x] **Step 3: Create `src/formatter/pointer_style.rs`**

```rust
use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::{FormatConfig, PointerAlignment};
use regex::Regex;
use std::sync::LazyLock;

const TYPE_KEYWORDS: &str = r"(?i)\b(int|char|void|long|short|float|double|unsigned|signed|const|volatile|static|extern|struct|enum|union|bool|auto|register|restrict|inline|size_t|...|FILE)";

pub fn fix_pointer_style(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Processes lines, skipping preprocessor/comments/block comments.
    // Uses split_outside_strings and split_outside_inline_block_comments.
    // Normalizes pointer declarations to `type* name` first,
    // then applies left (int* p) or right (int *p) alignment.
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`. Comprehensive type keyword list. Handles inline block comments.

- [x] **Step 4: Register module, update pipeline**

Add `pub mod pointer_style;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
pointer_style::fix_pointer_style(source, config)?;
```

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): pointer alignment style (left/right)"
```
