### Task 06: Blank Line Normalization

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/blank_lines.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::blank_lines::fix_blank_lines;

#[test]
fn test_max_consecutive_blank_lines() {
    let input = "int x;\n\n\n\n\nint y;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_blank_lines(&src, 2, 1);
    assert_eq!(result.content, "int x;\n\n\nint y;\n");
}

#[test]
fn test_blank_lines_after_include() {
    let input = "#include <stdio.h>\n#include <stdlib.h>\nint main() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_blank_lines(&src, 2, 1);
    assert!(result.content.contains("#include <stdlib.h>\n\nint main()"));
}

#[test]
fn test_no_blank_lines_at_file_start() {
    let input = "\n\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_blank_lines(&src, 2, 1);
    assert!(result.content.starts_with("int x;"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_max_consecutive test_blank_lines_after test_no_blank_lines_at_file_start`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/blank_lines.rs`**

```rust
use crate::common::source::SourceFile;
use std::path::PathBuf;

pub fn fix_blank_lines(source: &SourceFile, max_blank: usize, after_include: usize) -> SourceFile {
    let mut lines: Vec<String> = source.lines.clone();
    lines = collapse_blank_lines(&lines, max_blank);
    lines = ensure_blank_after_includes(&lines, after_include);
    lines = trim_leading_blanks(&lines);
    let content = lines.join("\n");
    let has_newline = source.content.ends_with('\n');
    let final_content = if has_newline && !content.is_empty() {
        format!("{}\n", content)
    } else {
        content
    };
    SourceFile::from_string(&final_content, source.path.clone())
}

fn collapse_blank_lines(lines: &[String], max: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut consecutive = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            consecutive += 1;
            if consecutive <= max {
                result.push(line.clone());
            }
        } else {
            consecutive = 0;
            result.push(line.clone());
        }
    }
    result
}

fn ensure_blank_after_includes(lines: &[String], count: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut last_was_include = false;
    let mut blanks_since_include = 0usize;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('#') && trimmed.contains("include") {
            if last_was_include {
                blanks_since_include = 0;
            }
            result.push(line.clone());
            last_was_include = true;
            continue;
        }
        if last_was_include {
            if trimmed.is_empty() {
                blanks_since_include += 1;
                if blanks_since_include <= count {
                    result.push(line.clone());
                }
            } else {
                for _ in blanks_since_include..count {
                    result.push(String::new());
                }
                result.push(line.clone());
                last_was_include = false;
                blanks_since_include = 0;
            }
        } else {
            result.push(line.clone());
        }
    }
    result
}

fn trim_leading_blanks(lines: &[String]) -> Vec<String> {
    let mut start = 0;
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            start = i;
            break;
        }
    }
    lines[start..].to_vec()
}
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod blank_lines;` to `src/formatter/mod.rs`. Update `format_source` to include:

```rust
let source = blank_lines::fix_blank_lines(
    &source,
    config.format.max_consecutive_blank_lines.unwrap_or(2),
    config.format.blank_lines_after_include.unwrap_or(1),
);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): blank line normalization"
```
