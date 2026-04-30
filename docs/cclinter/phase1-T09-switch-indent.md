### Task 09: switch-case Indentation

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/switch_indent.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

Note: File named `switch_indent.rs` (not `switch-case.rs` — no hyphens in Rust module names).

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::switch_indent::fix_switch_indent;

#[test]
fn test_case_indent_enabled() {
    let input = "switch (x) {\ncase 1:\nbreak;\ncase 2:\nbreak;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_switch_indent(&src, true);
    assert!(result.content.contains("  case 1:"));
    assert!(result.content.contains("    break;"));
}

#[test]
fn test_case_indent_disabled() {
    let input = "  case 1:\n    break;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_switch_indent(&src, false);
    assert!(result.content.contains("case 1:"));
    assert!(result.content.contains("  break;"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_case_indent`
Expected: FAIL.

- [x] **Step 3: Create `src/formatter/switch_indent.rs`**

```rust
use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;
use regex::Regex;
use std::sync::LazyLock;

pub fn fix_switch_indent(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.switch_case_indent { return Ok(()); }
    let indent_width = config.indent_width;
    // Tracks switch_stack: Vec<(entry_depth, switch_indent)>
    // case labels indented to switch_indent + indent_width
    // case body indented to switch_indent + 2 * indent_width
    // Handles nested switches, reindenting as needed.
    // Uses count_braces_outside_strings for accurate brace tracking.
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`. Supports nested switches. Uses `split_outside_strings` for accurate counting.

- [x] **Step 4: Register module, update pipeline**

Add `pub mod switch_indent;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
switch_indent::fix_switch_indent(source, config)?;
```

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): switch-case indentation"
```
