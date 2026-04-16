### Task 02: Encoding / Line Ending / Trailing Whitespace

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/encoding.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`
- Test fixtures: `tools/linter/cclinter/tests/fixtures/input/encoding_test.c`
- Test fixtures: `tools/linter/cclinter/tests/fixtures/expected/encoding_test.c`

- [ ] **Step 1: Create test fixture — input**

Create `tests/fixtures/input/encoding_test.c` with mixed issues:

```c
int main() {\t
    printf("hello");\r\n
    return 0;   \r\n}
```

(Use raw bytes: contains CRLF, tab, trailing spaces.)

- [ ] **Step 2: Create test fixture — expected**

Create `tests/fixtures/expected/encoding_test.c`:

```c
int main() {
    printf("hello");
    return 0;
}
```

- [ ] **Step 3: Write failing test in `tests/formatter_tests.rs`**

All test functions below assume these imports at file top:

```rust
use cclinter::common::source::SourceFile;
use cclinter::formatter::encoding::fix_encoding;
use std::path::PathBuf;

#[test]
fn test_strip_trailing_whitespace() {
    let input = "int x = 1;   \nint y = 2;\t\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_encoding(&src);
    assert_eq!(result.content, "int x = 1;\nint y = 2;\n");
}

#[test]
fn test_crlf_to_lf() {
    let input = "line1\r\nline2\r\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_encoding(&src);
    assert_eq!(result.content, "line1\nline2\n");
}

#[test]
fn test_remove_bom() {
    let input = "\u{feff}int x;";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_encoding(&src);
    assert_eq!(result.content, "int x;");
}

#[test]
fn test_combined_encoding_fixes() {
    let input = "\u{feff}int x = 1;   \r\nint y = 2;\t\r\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_encoding(&src);
    assert_eq!(result.content, "int x = 1;\nint y = 2;\n");
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cargo test --test formatter_tests`
Expected: FAIL — `fix_encoding` does not exist yet.

- [ ] **Step 5: Create `src/formatter/encoding.rs`**

```rust
use crate::common::source::SourceFile;
use std::path::PathBuf;

pub fn fix_encoding(source: &SourceFile) -> SourceFile {
    let content = source.content.as_str();
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let content = content.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<String> = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    let result = lines.join("\n");
    let has_newline = content.ends_with('\n');
    let final_content = if has_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    SourceFile::from_string(&final_content, source.path.clone())
}
```

- [ ] **Step 6: Register module in `src/formatter/mod.rs`**

Add `pub mod encoding;` to `src/formatter/mod.rs`.

Update `format_source` to call `fix_encoding`:

```rust
pub mod encoding;

use crate::common::source::SourceFile;

pub fn format_source(source: &SourceFile) -> SourceFile {
    let source = encoding::fix_encoding(source);
    source
}
```

- [ ] **Step 7: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All 4 tests PASS.

- [ ] **Step 8: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): UTF-8 BOM removal, CRLF→LF, trailing whitespace strip"
```
