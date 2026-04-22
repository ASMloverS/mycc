### Task 06: Blank Line Normalization

**Files:**
- Create: `tools/linter/cclinter/src/formatter/blank_lines.rs`
- Already registered: `tools/linter/cclinter/src/formatter/mod.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs`:

```rust
use cclinter::formatter::blank_lines::fix_blank_lines;

#[test]
fn test_blank_collapse_consecutive() {
    let mut config = FormatConfig::default();
    config.max_consecutive_blank_lines = 2;
    let mut src = SourceFile::from_string("int x;\n\n\n\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\n\nint y;\n");
}

#[test]
fn test_blank_collapse_to_one() {
    let mut config = FormatConfig::default();
    config.max_consecutive_blank_lines = 1;
    let mut src = SourceFile::from_string("int x;\n\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\nint y;\n");
}

#[test]
fn test_blank_after_include() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 1;
    let mut src = SourceFile::from_string("#include <stdio.h>\n#include <stdlib.h>\nint main() {}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("#include <stdlib.h>\n\nint main()"));
}

#[test]
fn test_blank_after_include_two() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 2;
    let mut src = SourceFile::from_string("#include <stdio.h>\nint x;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("#include <stdio.h>\n\n\nint x;"));
}

#[test]
fn test_blank_leading_removed() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("\n\nint x;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.starts_with("int x;"));
}

#[test]
fn test_blank_trailing_removed() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n\n\n\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.ends_with("int x;\n"));
}

#[test]
fn test_blank_before_function() {
    let mut config = FormatConfig::default();
    config.blank_lines_before_function = 1;
    let mut src = SourceFile::from_string("int x;\nvoid f() {\n}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("int x;\n\nvoid f()"));
}

#[test]
fn test_blank_no_change_needed() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\nint y;\n");
}

#[test]
fn test_blank_empty_input() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_blank_only_whitespace_lines() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n   \n   \nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\nint y;\n");
}

#[test]
fn test_blank_include_block_multiple_groups() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 1;
    let mut src = SourceFile::from_string(
        "#include <stdio.h>\n#include <stdlib.h>\n\n#include \"my.h\"\nint x;\n",
        PathBuf::from("test.c"),
    );
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("#include \"my.h\"\n\nint x;"));
}

#[test]
fn test_blank_preserve_single_newline_ending() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n");
}

#[test]
fn test_blank_no_trailing_newline_input() {
    let mut config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;");
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_blank_`
Expected: FAIL.

- [x] **Step 3: Implement `src/formatter/blank_lines.rs`**

```rust
pub fn fix_blank_lines(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation with: trim leading, normalize whitespace, collapse consecutive,
    // ensure after includes, ensure before functions, trim trailing.
}
```

- [x] **Step 4: Already registered in pipeline**

`src/formatter/mod.rs` already contains `pub mod blank_lines;` and calls `blank_lines::fix_blank_lines(source, config)?;`.

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Build**

Run: `cargo build`
Expected: Success.
