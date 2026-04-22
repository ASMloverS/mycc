### Task 05: Brace Style — Attach / Breakout / Hybrid

**Files:**
- Modify: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/formatter/braces.rs`
- Test: `tools/linter/cclinter/tests/formatter_tests.rs`

- [x] **Step 1: Write failing tests**

Add to `tests/formatter_tests.rs`:

```rust
use cclinter::formatter::braces::fix_braces;

#[test]
fn test_brace_attach_function() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("void f()\n{\n  return;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "void f() {\n  return;\n}\n");
}

#[test]
fn test_brace_attach_if() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("if (x)\n{\n  y();\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "if (x) {\n  y();\n}\n");
}

#[test]
fn test_brace_attach_else() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("if (x) {\n} else\n{\n  y();\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.contains("} else {"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test formatter_tests test_brace_attach`
Expected: FAIL (module does not exist yet).

- [x] **Step 3: Create `src/formatter/braces.rs`**

```rust
use crate::common::source::SourceFile;
use crate::config::{BraceStyle, FormatConfig};

pub fn fix_braces(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.brace_style {
        BraceStyle::Attach => {
            source.content = attach_braces(&source.content);
        }
        BraceStyle::Breakout => {
            source.content = breakout_braces(&source.content);
        }
        BraceStyle::AttachBreakout => {
            source.content = attach_breakout_hybrid(&source.content);
        }
    }
    Ok(())
}
```

Helpers: `is_protected_line`, `brace_in_string_literal`, `attach_braces`,
`breakout_braces`, `find_attach_brace`, `breakout_type_keywords`.

- [x] **Step 4: Register module, update pipeline**

Add `pub mod braces;` to `src/formatter/mod.rs`. Call in `format_source`:

```rust
braces::fix_braces(&mut source, config)?;
```

- [x] **Step 5: Run tests**

Run: `cargo test --test formatter_tests`
Expected: All tests PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): brace style attach/breakout/hybrid"
```
