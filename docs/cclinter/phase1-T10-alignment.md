### Task 10: Continuation Alignment + Struct/Enum Alignment

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/alignment.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::alignment::fix_alignment;

#[test]
fn test_struct_field_alignment() {
    let input = "struct Foo {\nint x;\nchar *name;\nfloat value;\n};\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_alignment(&src);
    assert!(result.content.contains("int   x;"));
    assert!(result.content.contains("char*  name;"));
    assert!(result.content.contains("float value;"));
}

#[test]
fn test_enum_value_alignment() {
    let input = "enum Bar {\nFOO = 1,\nBAZ = 2,\nLONG_NAME = 3,\n};\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_alignment(&src);
    assert!(result.content.contains("FOO       = 1"));
    assert!(result.content.contains("LONG_NAME = 3"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_struct_field test_enum_value`
Expected: FAIL.

- [x] **Step 3: Create `src/formatter/alignment.rs`**

```rust
use crate::common::source::SourceFile;
use crate::common::string_utils::split_outside_strings;
use crate::config::FormatConfig;
use regex::Regex;
use std::sync::LazyLock;

pub fn fix_alignment(
    source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Detects struct/enum blocks using STRUCT_OPEN_RE / ENUM_OPEN_RE.
    // For struct: parses type + name from field lines, aligns names column.
    // For enum: aligns `=` signs for enum members with values.
    // Uses strip_trailing_comment to separate code from inline comments.
    // Handles nested blocks (struct inside struct, enum inside struct).
    // Requires >= 2 fields in a block before aligning.
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`. Handles both struct field alignment (type column) and enum member alignment (`=` column). Preserves inline comments. Handles nested blocks.

- [x] **Step 4: Register module, update pipeline**

Add `pub mod alignment;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
alignment::fix_alignment(source, config)?;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): struct/enum field alignment"
```
