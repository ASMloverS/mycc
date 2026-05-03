# Task 08: Formatter — indent (AST-aware)

> Status: ✅ Done
> Depends: Task 05
> Output: Python indent unified to configured `indent_width` spaces (default 4)

## Goal

CST/AST-based indent formatting:
- Unify spaces or tabs (config `use_tabs`)
- Configurable `indent_width` (default 4)
- Use Python tokenizer INDENT/DEDENT tokens → indent depth
- Rebuild each line's indent prefix

## Challenges

Python indent = syntax. Tokenizer emits INDENT/DEDENT tokens → use these to rebuild indent.

Multi-line structures (implicit line continuation inside brackets) ignore INDENT/DEDENT. This task handles INDENT/DEDENT only. Continuation indent deferred.

## Steps

### 1. formatter/indent.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;
use crate::cst::{CSTSource, CSTLine, IndentInfo};

pub fn fix_indent(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let indent_width = config.indent_width;
    let use_tabs = config.use_tabs;

    let had_newline = source.content.ends_with('\n');
    let lines: Vec<&str> = source.content.lines().collect();

    // 方法: 利用 tokenizer 的 INDENT/DEDENT 追踪层级
    let tokens = crate::cst::tokens::tokenize_source(&source.content)?;
    let mut depth: usize = 0;
    let mut result: Vec<String> = Vec::with_capacity(lines.len());

    // 构建 line -> indent_depth 映射
    let indent_changes = compute_indent_depths(&tokens, lines.len());

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        if trimmed.is_empty() {
            result.push(String::new());
            continue;
        }

        // DEDENT 发生在当前行之前, 先减少 depth
        if let Some(dedent_count) = indent_changes.get(&i) {
            depth = depth.saturating_sub(*dedent_count);
        }

        let indent_str = if use_tabs {
            "\t".repeat(depth)
        } else {
            " ".repeat(depth * indent_width)
        };

        result.push(format!("{}{}", indent_str, trimmed));

        // INDENT 发生在当前行之后 (下一行开始缩进)
        // 在 compute_indent_depths 中处理
    }

    let out = result.join("\n");
    source.content = if had_newline && !out.is_empty() {
        format!("{}\n", out)
    } else {
        out
    };
    Ok(())
}

fn compute_indent_depths(tokens: &[LocatedToken], line_count: usize) -> HashMap<usize, usize> {
    // 遍历 tokens:
    // - 遇到 INDENT: 记录下一行的 depth 增加
    // - 遇到 DEDENT: 记录当前行 (或下一行) 的 depth 减少
    // 返回 HashMap<line_num, dedent_count>
}
```

## Tests

```rust
#[test]
fn normalize_indent_to_4_spaces() {
    // 输入: 2 空格缩进
    let input = "if True:\n  pass\n";
    let expected = "if True:\n    pass\n";
    // ...
}

#[test]
fn preserve_4_space_indent() {
    let input = "if True:\n    pass\n";
    // 应不变
}

#[test]
fn nested_indent() {
    let input = "if True:\n  if True:\n    pass\n";
    let expected = "if True:\n    if True:\n        pass\n";
}

#[test]
fn tab_to_spaces() {
    let config = FormatConfig { use_tabs: false, indent_width: 4, ..Default::default() };
    let input = "if True:\n\tpass\n";
    let expected = "if True:\n    pass\n";
}

#[test]
fn indent_width_2() {
    let config = FormatConfig { indent_width: 2, ..Default::default() };
    let input = "if True:\n    pass\n";
    let expected = "if True:\n  pass\n";
}

#[test]
fn dedent_correct() {
    let input = "if True:\n    x = 1\ny = 2\n";
    // dedent 后 y = 2 应在顶层
}
```

## Verify

```bash
cargo test -- indent
```
