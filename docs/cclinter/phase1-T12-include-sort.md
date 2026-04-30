### Task 12: #include Google Three-Group Sorting

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/include_sort.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::include_sort::fix_include_sort;

#[test]
fn test_sort_system_headers() {
    let input = "#include <stdio.h>\n#include <stdlib.h>\n#include <assert.h>\n\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_include_sort(&src);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines[0].contains("<assert.h>"));
    assert!(lines[1].contains("<stdio.h>"));
    assert!(lines[2].contains("<stdlib.h>"));
}

#[test]
fn test_sort_project_headers() {
    let input = "#include \"foo.h\"\n#include \"bar.h\"\n#include \"baz.h\"\n\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_include_sort(&src);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines[0].contains("\"bar.h\""));
    assert!(lines[1].contains("\"baz.h\""));
    assert!(lines[2].contains("\"foo.h\""));
}

#[test]
fn test_three_group_sort() {
    let input = "#include \"foo.h\"\n#include <stdio.h>\n#include \"foo.c\"\n#include <stdlib.h>\n\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_include_sort(&src);
    let lines: Vec<&str> = result.content.lines().collect();
    let mut saw_system = false;
    let mut saw_project = false;
    for line in lines {
        if line.starts_with("#include <") {
            saw_system = true;
            assert!(!saw_project, "system headers must come before project headers");
        }
        if line.starts_with("#include \"") {
            saw_project = true;
        }
    }
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_sort_system test_sort_project test_three_group_sort`
Expected: FAIL.

- [x] **Step 3: Create `src/formatter/include_sort.rs`**

```rust
use crate::common::source::SourceFile;
use crate::config::{FormatConfig, IncludeSorting};
use regex::Regex;
use std::sync::LazyLock;

pub fn fix_include_sort(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.include_sorting == IncludeSorting::Disabled { return Ok(()); }
    // Three groups: corresponding (matching file stem), system (<...>), project ("...")
    // Skips sorting if conditional PP directives (#if, #ifdef, etc.) found in include block.
    // Skips if non-include, non-blank lines found between first and last include.
    // Each group sorted alphabetically by header path.
    // Blank line separator between groups.
    Ok(())
}
```

Key: takes `&mut SourceFile` + `&FormatConfig`. Three groups: corresponding header → system → project. Skips if conditional PP directives present. Uses file stem matching for "corresponding" group.

- [x] **Step 4: Register module, update pipeline**

Add `pub mod include_sort;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
include_sort::fix_include_sort(source, config)?;
```

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): #include Google three-group sorting"
```
