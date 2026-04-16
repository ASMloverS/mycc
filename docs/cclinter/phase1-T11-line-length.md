### Task 11: Line Length Wrapping — 120 Columns

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/line_length.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_wrap_long test_no_wrap_short test_wrap_preserves_indent`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/line_length.rs`**

```rust
use crate::common::source::SourceFile;
use std::path::PathBuf;

pub fn fix_line_length(source: &SourceFile, column_limit: usize) -> SourceFile {
    let indent_width = 2;
    let lines: Vec<String> = source
        .lines
        .iter()
        .flat_map(|line| {
            if line.len() <= column_limit {
                return vec![line.clone()];
            }
            wrap_line(line, column_limit, indent_width)
        })
        .collect();
    let content = lines.join("\n");
    let has_newline = source.content.ends_with('\n');
    let final_content = if has_newline && !content.is_empty() {
        format!("{}\n", content)
    } else {
        content
    };
    SourceFile::from_string(&final_content, source.path.clone())
}

fn wrap_line(line: &str, limit: usize, indent_width: usize) -> Vec<String> {
    let leading_ws: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let base_indent = format!("{}{}", leading_ws, " ".repeat(indent_width * 2));
    let mut result = vec![line.to_string()];
    let mut current = line.to_string();
    while current.len() > limit {
        let break_pos = find_break_point(&current, limit);
        if break_pos <= leading_ws.len() + 4 {
            break;
        }
        let before = current[..break_pos].trim_end().to_string();
        let after = format!("{}{}", base_indent, current[break_pos..].trim_start());
        result.clear();
        result.push(before);
        if after.len() > limit {
            result.extend(wrap_line(&after, limit, indent_width));
        } else {
            result.push(after);
        }
        current = after;
        break;
    }
    result
}

fn find_break_point(line: &str, limit: usize) -> usize {
    let candidates = [',', ' ', ';', '(', ')', '|', '&'];
    let mut best = 0;
    for (i, ch) in line.char_indices() {
        if i >= limit {
            break;
        }
        if candidates.contains(&ch) {
            best = i + 1;
        }
    }
    if best > 0 { best } else { limit }
}
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod line_length;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = line_length::fix_line_length(
    &source,
    config.format.column_limit.unwrap_or(120),
);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): line length wrapping at configurable column limit"
```
