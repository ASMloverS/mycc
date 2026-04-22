### Task 07: Comment Conversion — `/* */` → `//`

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/comments.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::comments::fix_comments;
use cclinter::config::CommentStyle;
```

Tests added: `test_comment_single_line_block`, `test_comment_standalone_block`,
`test_comment_multi_line_block`, `test_comment_preserve_double_slash`,
`test_comment_copyright_block`, `test_comment_preserve_mode`,
`test_comment_string_literal_not_converted`, `test_comment_empty_block`,
`test_comment_adjacent_blocks`, `test_comment_empty_input`,
`test_comment_multi_line_with_stars`, `test_comment_inline_preserves_code`.

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_comment_`
Expected: FAIL. Result: 8 failed, 4 passed (stub returns Ok).

- [x] **Step 3: Implement `src/formatter/comments.rs`**

Character-scanning approach to handle string literals, char literals,
and `//` line comments correctly. Converts `/* */` blocks to `//` lines,
strips leading `*` from continuation lines.

```rust
use crate::common::source::SourceFile;
use crate::config::{CommentStyle, FormatConfig};

pub fn fix_comments(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.comment_style != CommentStyle::DoubleSlash {
        return Ok(());
    }
    if source.content.is_empty() {
        return Ok(());
    }
    source.content = convert_block_comments(&source.content);
    Ok(())
}
```

- [x] **Step 4: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All 86 tests PASS (74 existing + 12 new).

- [x] **Step 5: Build**

Run: `cargo build`
Expected: 0 warnings.
