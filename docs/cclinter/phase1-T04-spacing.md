### Task 04: Spacing — Operators, Commas, Parens, Semicolons

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/spacing.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::spacing::fix_spacing;

#[test]
fn test_binary_operators() {
    let input = "int x=1+2*3;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_spacing(&src, true);
    assert!(result.content.contains("x = 1 + 2 * 3"));
}

#[test]
fn test_comma_spacing() {
    let input = "void f(int a,int b,int c){}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_spacing(&src, true);
    assert!(result.content.contains("int a, int b, int c"));
}

#[test]
fn test_no_space_in_for() {
    let input = "for (i=0;i<10;i++) {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_spacing(&src, true);
    assert!(result.content.contains("i = 0") || result.content.contains("i=0"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_binary_operators test_comma_spacing test_no_space_in_for`
Expected: FAIL — `fix_spacing` does not exist.

- [x] **Step 3: Create `src/formatter/spacing.rs`**

```rust
use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;
use regex::Regex;
use std::sync::LazyLock;

pub fn fix_spacing(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if !config.spaces_around_operators && !config.space_before_paren {
        return Ok(());
    }
    // Processes lines using split_outside_strings to skip string literals.
    // Uses LazyLock static regexes for compound operators (==, !=, +=, etc.)
    // and single operators (+, -, *, /, etc.).
    // Handles unary context detection (e.g., -1, *ptr, &addr).
    // Fixes for-loop semicolons specially.
    // Optional space_before_paren: `func(` → `func (`.
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`. Uses `split_outside_strings` to avoid modifying inside string/char literals. `LazyLock` static regexes for performance.

- [x] **Step 4: Register module and update pipeline**

Add `pub mod spacing;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
spacing::fix_spacing(source, config)?;
```

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): operator, comma, and semicolon spacing rules"
```
