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

- [ ] **Step 3: Create `src/formatter/alignment.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_alignment(source: &SourceFile) -> SourceFile {
    let lines: Vec<String> = source.lines.clone();
    let lines = align_block(&lines, "struct", "}");
    let lines = align_block(&lines, "enum", "}");
    let content = lines.join("\n");
    let has_newline = source.content.ends_with('\n');
    let final_content = if has_newline && !content.is_empty() {
        format!("{}\n", content)
    } else {
        content
    };
    SourceFile::from_string(&final_content, source.path.clone())
}

fn align_block(lines: &[String], keyword: &str, end_marker: &str) -> Vec<String> {
    let mut result = lines.to_vec();
    let kw_re = Regex::new(&format!(r"^\s*{}\s+\w+\s*\{{", keyword)).unwrap();
    let mut i = 0;
    while i < result.len() {
        if kw_re.is_match(&result[i]) {
            let start = i + 1;
            let mut end = start;
            while end < result.len() && !result[end].trim().starts_with(end_marker) {
                end += 1;
            }
            if end > start {
                let aligned = align_fields(&result[start..end]);
                for (j, line) in aligned.iter().enumerate() {
                    result[start + j] = line.clone();
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    result
}

fn align_fields(fields: &[String]) -> Vec<String> {
    let field_re = Regex::new(r"^(\s*)(\S+)\s+(\S+)(.*)$").unwrap();
    let type_re = Regex::new(r"^(\s*)(\S+\*?)\s+(\S+)(.*)$").unwrap();
    let mut max_type_len = 0usize;
    let parsed: Vec<Option<(String, String, String, String)>> = fields
        .iter()
        .map(|line| {
            if let Some(caps) = type_re.captures(line) {
                let type_len = caps[2].len();
                if type_len > max_type_len {
                    max_type_len = type_len;
                }
                Some((
                    caps[1].to_string(),
                    caps[2].to_string(),
                    caps[3].to_string(),
                    caps[4].to_string(),
                ))
            } else {
                None
            }
        })
        .collect();

    parsed
        .iter()
        .zip(fields.iter())
        .map(|(p, original)| match p {
            Some((indent, type_name, name, rest)) => {
                let padding = max_type_len - type_name.len();
                format!("{}{}{}{}{}", indent, type_name, " ".repeat(padding + 1), name, rest)
            }
            None => original.clone(),
        })
        .collect()
}
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod alignment;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = alignment::fix_alignment(&source);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): struct/enum field alignment"
```
