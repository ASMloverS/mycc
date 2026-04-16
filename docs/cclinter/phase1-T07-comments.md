### Task 07: Comment Conversion — `/* */` → `//`

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/comments.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs` (imports assumed from T02):

```rust
use cclinter::formatter::comments::fix_comments;

#[test]
fn test_single_line_block_comment() {
    let input = "int x; /* comment */ int y;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_comments(&src);
    assert!(result.content.contains("// comment"));
}

#[test]
fn test_standalone_block_comment() {
    let input = "/* standalone comment */\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_comments(&src);
    assert!(result.content.starts_with("// standalone comment"));
}

#[test]
fn test_multi_line_block_comment() {
    let input = "/* line1\n   line2\n   line3 */\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_comments(&src);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines[0].starts_with("//"));
    assert!(lines[1].trim().starts_with("//"));
    assert!(lines[2].trim().starts_with("//"));
}

#[test]
fn test_preserve_double_slash() {
    let input = "// already slash comment\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_comments(&src);
    assert_eq!(result.content, input);
}

#[test]
fn test_copyright_block() {
    let input = "/* Copyright 2026 My Corp\n * All rights reserved. */\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let result = fix_comments(&src);
    assert!(result.content.contains("// Copyright 2026 My Corp"));
    assert!(result.content.contains("// All rights reserved."));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_single_line_block test_standalone_block test_multi_line_block test_preserve_double_slash test_copyright_block`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/comments.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_comments(source: &SourceFile) -> SourceFile {
    let mut content = source.content.clone();
    let block_re = Regex::new(r"/\*([\s\S]*?)\*/").unwrap();
    content = block_re
        .replace_all(&content, |caps: &regex::Captures| {
            let body = &caps[1];
            let lines: Vec<&str> = body.lines().collect();
            if lines.len() <= 1 {
                let text = body.trim().trim_start_matches('*').trim();
                return format!("// {}", text);
            }
            let converted: Vec<String> = lines
                .iter()
                .map(|line| {
                    let trimmed = line.trim().trim_start_matches('*').trim();
                    if trimmed.is_empty() {
                        "//".to_string()
                    } else {
                        format!("// {}", trimmed)
                    }
                })
                .collect();
            converted.join("\n")
        })
        .to_string();
    SourceFile::from_string(&content, source.path.clone())
}
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod comments;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = comments::fix_comments(&source);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): block comment to double-slash conversion"
```
