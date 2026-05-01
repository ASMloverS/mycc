# Task 09: Formatter — blank_lines (AST-aware)

> Status: ⬜ Not started
> Depends: Task 05, Task 08
> Output: PEP 8 blank line rules: 2 blank lines before top-level fn/class, 1 blank line before methods inside class

## Goal

Normalize blank lines in Python source:
- Top-level fn/class def: 2 blank lines before
- Method def inside class: 1 blank line before
- Max consecutive blank lines: configurable (`max_consecutive_blank_lines`)
- Strip leading/trailing blank lines

## Reference

- `cclinter/src/formatter/blank_lines.rs` — structural reference
- PEP 8 blank line rules

## PEP 8 Rules

- 2 blank lines surround top-level fn/class defs
- 1 blank line surrounds method defs inside class
- Blank lines inside fn/method separate logical blocks, use sparingly
- File ends with single newline

## Steps

### 1. formatter/blank_lines.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;
use crate::cst::AstNodeKind;

pub fn fix_blank_lines(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if source.content.is_empty() {
        return Ok(());
    }

    let had_newline = source.content.ends_with('\n');
    let mut lines: Vec<String> = source.content.lines().map(|l| l.to_string()).collect();

    // Step 1: strip leading blank lines
    lines = trim_leading_blanks(&lines);

    // Step 2: normalize whitespace-only lines to empty
    lines = normalize_whitespace_lines(&lines);

    // Step 3: collapse consecutive blank lines
    lines = collapse_blank_lines(&lines, config.max_consecutive_blank_lines);

    // Step 4: mark AST node positions from parse result
    let ast_marks = mark_ast_positions(&source.content);

    // Step 5: ensure 2 blank lines before top-level class/function def
    lines = ensure_blank_before_toplevel(&lines, &ast_marks, config.blank_lines_before_class, config.blank_lines_before_function);

    // Step 6: ensure 1 blank line before method def inside class
    lines = ensure_blank_before_methods(&lines, &ast_marks, config.blank_lines_inside_class);

    // Step 7: strip trailing blank lines
    lines = trim_trailing_blanks(&lines);

    let mut result = lines.join("\n");
    if had_newline && !result.is_empty() {
        result.push('\n');
    }
    source.content = result;
    Ok(())
}
```

### 2. AST position marking

```rust
struct AstMark {
    line: usize,
    kind: AstMarkKind,
    indent_level: usize,
}

enum AstMarkKind {
    FunctionDef,
    ClassDef,
}

fn mark_ast_positions(source: &str) -> Vec<AstMark> {
    // rustpython_parser::parse → AST
    // walk AST → record FunctionDef/ClassDef line numbers + indent level
    // top-level = indent_level 0, method inside class = indent_level 1
}
```

## Tests

```rust
#[test]
fn blank_lines_before_toplevel_function() {
    let input = "import os\ndef foo():\n    pass\n";
    let expected = "import os\n\n\ndef foo():\n    pass\n";
}

#[test]
fn blank_lines_before_toplevel_class() {
    let input = "x = 1\nclass Foo:\n    pass\n";
    let expected = "x = 1\n\n\nclass Foo:\n    pass\n";
}

#[test]
fn blank_lines_inside_class() {
    let input = concat!(
        "class Foo:\n",
        "    def bar(self):\n",
        "        pass\n",
        "    def baz(self):\n",
        "        pass\n",
    );
    let expected = concat!(
        "class Foo:\n",
        "    def bar(self):\n",
        "        pass\n",
        "\n",
        "    def baz(self):\n",
        "        pass\n",
    );
}

#[test]
fn collapse_consecutive_blanks() {
    // 4 blank lines → collapse to 2
}

#[test]
fn trim_leading_trailing_blanks() {
    let input = "\n\n\nx = 1\n\n\n";
    let expected = "x = 1\n";
}
```

## Verify

```bash
cargo test -- blank_lines
```
