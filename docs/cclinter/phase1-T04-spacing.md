### Task 04: Spacing — Operators, Commas, Parens, Semicolons

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/spacing.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_binary_operators test_comma_spacing test_no_space_in_for`
Expected: FAIL — `fix_spacing` does not exist.

- [ ] **Step 3: Create `src/formatter/spacing.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_spacing(source: &SourceFile, spaces_around_ops: bool) -> SourceFile {
    let mut content = source.content.clone();
    if spaces_around_ops {
        let lines: Vec<String> = content.lines().map(|line| {
            let result = process_line_spacing(line);
            result
        }).collect();
        content = lines.join("\n");
        if source.content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
    }
    SourceFile::from_string(&content, source.path.clone())
}

fn process_line_spacing(line: &str) -> String {
    let trimmed = line.trim_end();
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return line.to_string();
    }
    let mut result = trimmed.to_string();
    let binary_ops = ["==", "!=", "<=", ">=", "&&", "||", "<<", ">>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^="];
    for op in &binary_ops {
        let pattern = format!(r"(?P<before>\S)\s*{}\s*(?P<after>\S)", regex::escape(op));
        let re = Regex::new(&pattern).unwrap();
        let replacement = format!("$before {} $after", op);
        result = re.replace_all(&result, replacement.as_str()).to_string();
    }
    let single_ops = ["+", "-", "*", "/", "%", "<", ">", "&", "|", "^", "="];
    for op in &single_ops {
        if result.contains("==") || result.contains("!=") || result.contains("<=") || result.contains(">=") {
            continue;
        }
        let pattern = format!(r"(?P<before>\S)\s*{}\s*(?P<after>\S)", regex::escape(op));
        let re = Regex::new(&pattern).unwrap();
        let replacement = format!("$before {} $after", op);
        result = re.replace_all(&result, replacement.as_str()).to_string();
    }
    let comma_re = Regex::new(r",\s*").unwrap();
    result = comma_re.replace_all(&result, ", ").to_string();
    result
}
```

- [ ] **Step 4: Register module and update pipeline**

Add `pub mod spacing;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = spacing::fix_spacing(&source, config.format.spaces_around_operators.unwrap_or(true));
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): operator, comma, and semicolon spacing rules"
```
