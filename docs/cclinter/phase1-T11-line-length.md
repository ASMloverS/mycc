### Task 11: Line Length Wrapping — 120 Columns

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/line_length.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::line_length::fix_line_length;

#[test]
fn test_wrap_long_line() {
    let input = "int very_long_variable_name = some_function_with_many_args(arg1, arg2, arg3, arg4, arg5, arg6, arg7);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_line_length(&src, 80);
    for line in result.content.lines() {
        assert!(line.len() <= 82, "Line too long: {} (len={})", line, line.len());
    }
}

#[test]
fn test_no_wrap_short_line() {
    let input = "int x = 1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_line_length(&src, 80);
    assert_eq!(result.content, input);
}

#[test]
fn test_wrap_preserves_indent() {
    let input = "    int result = very_long_function_name_that_exceeds_the_column_limit_by_a_lot(a, b);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_line_length(&src, 60);
    let lines: Vec<&str> = result.content.lines().collect();
    for line in &lines[1..] {
        assert!(line.starts_with("    "), "Continuation should preserve base indent");
    }
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_wrap_long test_no_wrap_short test_wrap_preserves_indent`
Expected: FAIL.

- [x] **Step 3: Create `src/formatter/line_length.rs`**

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_line_length(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let limit = config.column_limit;
    let indent_width = config.indent_width;
    // 1. merge_continuations: joins lines that appear to be continuations
    //    (skips preprocessor, comments, block comments, lines ending with ; { } ) :)
    // 2. Wraps lines exceeding column_limit using break points
    // Break point priority: comma(3) > space/semicolon(2) > close-paren(1) > open-paren/bitops(0)
    // Avoids breaking inside string literals (tracked via string_spans)
    // Continuation indent = base indent + indent_width
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`. Merges continuation lines first, then wraps. Uses char-count (not byte-count) for column limit. Avoids breaking inside string literals.

- [x] **Step 4: Register module, update pipeline**

Add `pub mod line_length;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
line_length::fix_line_length(source, config)?;
```

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): line length wrapping at configurable column limit"
```
