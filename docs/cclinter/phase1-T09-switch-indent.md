### Task 09: switch-case Indentation

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/switch_indent.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

Note: File named `switch_indent.rs` (not `switch-case.rs` — no hyphens in Rust module names).

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_case_indent`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/switch_indent.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_switch_indent(source: &SourceFile, indent_case: bool) -> SourceFile {
    let mut in_switch = false;
    let mut brace_depth = 0i32;
    let switch_re = Regex::new(r"^\s*switch\s*\(").unwrap();
    let case_re = Regex::new(r"^(\s*)(case\s|default:)").unwrap();
    let indent_width = 2;

    let lines: Vec<String> = source
        .lines
        .iter()
        .map(|line| {
            let trimmed = line.trim();
            if switch_re.is_match(line) {
                in_switch = true;
                brace_depth = 0;
                return line.to_string();
            }
            if in_switch {
                brace_depth += trimmed.matches('{').count() as i32;
                brace_depth -= trimmed.matches('}').count() as i32;
                if brace_depth < 0 || (trimmed == "}" && brace_depth == 0) {
                    in_switch = false;
                    return line.to_string();
                }
                if indent_case {
                    if let Some(caps) = case_re.captures(line) {
                        let current_indent = caps[1].len();
                        let new_indent = " ".repeat(current_indent + indent_width);
                        let rest = &line[current_indent..];
                        return format!("{}{}", new_indent, rest);
                    }
                }
            }
            line.to_string()
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
```

- [ ] **Step 4: Register module, update pipeline**

Add `pub mod switch_indent;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = switch_indent::fix_switch_indent(
    &source,
    config.format.switch_case_indent.unwrap_or(true),
);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): switch-case indentation"
```
