### Task 12: #include Google Three-Group Sorting

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/include_sort.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_sort_system test_sort_project test_three_group_sort`
Expected: FAIL.

- [ ] **Step 3: Create `src/formatter/include_sort.rs`**

```rust
use crate::common::source::SourceFile;
use regex::Regex;
use std::path::PathBuf;

pub fn fix_include_sort(source: &SourceFile) -> SourceFile {
    let include_re = Regex::new(r#"^\s*#\s*include\s+([<"])([^>"]+)[>"]"#).unwrap();
    let mut corresponding: Vec<(String, String)> = vec![];
    let mut system: Vec<(String, String)> = vec![];
    let mut project: Vec<(String, String)> = vec![];
    let mut pre_include: Vec<String> = vec![];
    let mut post_include: Vec<String> = vec![];
    let mut in_includes = false;
    let mut past_includes = false;

    for line in &source.lines {
        if let Some(caps) = include_re.captures(line) {
            let delimiter = &caps[1];
            let header = &caps[2];
            let entry = (header.to_string(), line.trim().to_string());
            in_includes = true;
            if delimiter == "<" {
                system.push(entry);
            } else {
                let stem = source.path.file_stem().unwrap_or_default().to_string_lossy();
                if header.starts_with(&*stem) {
                    corresponding.push(entry);
                } else {
                    project.push(entry);
                }
            }
        } else if in_includes && line.trim().is_empty() && !past_includes {
            continue;
        } else if in_includes {
            past_includes = true;
            post_include.push(line.clone());
        } else {
            pre_include.push(line.clone());
        }
    }

    corresponding.sort_by(|a, b| a.0.cmp(&b.0));
    system.sort_by(|a, b| a.0.cmp(&b.0));
    project.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = pre_include;
    for (_, line) in &corresponding {
        result.push(line.clone());
    }
    if !corresponding.is_empty() && !system.is_empty() {
        result.push(String::new());
    }
    for (_, line) in &system {
        result.push(line.clone());
    }
    if !system.is_empty() && !project.is_empty() {
        result.push(String::new());
    }
    for (_, line) in &project {
        result.push(line.clone());
    }
    if !post_include.is_empty() {
        result.push(String::new());
        result.extend(post_include);
    }

    let content = result.join("\n");
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

Add `pub mod include_sort;` to `src/formatter/mod.rs`. Update `format_source`:

```rust
let source = include_sort::fix_include_sort(&source);
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): #include Google three-group sorting"
```
