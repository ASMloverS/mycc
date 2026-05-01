# Task 16: Checker — complexity 复杂度检查 (AST-based)

> 状态: ⬜ 未开始 | 依赖: Task 05 | 产出: fn/class/file line count + nesting depth check

## 目标

AST-based complexity checks:
- fn lines ≤ 50 (default)
- class lines ≤ 300 (default)
- file lines ≤ 1000 (default)
- nesting depth ≤ 4 (default)

## 参照

- `cclinter/src/checker/complexity.rs` — mirror structure + patterns

## 步骤

### 1. checker/complexity.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::ComplexityConfig;

pub fn check_complexity(source: &SourceFile, config: &ComplexityConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let lines = source.lines();

    // 文件行数
    if lines.len() > config.max_file_lines {
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            1, 1,
            Severity::Warning,
            "readability-file-size",
            &format!("File has {} lines (max {})", lines.len(), config.max_file_lines),
            lines.first().unwrap_or(&""),
        ));
    }

    diags.extend(check_function_lengths(&lines, source, config.max_function_lines));
    diags.extend(check_class_lengths(&lines, source, config.max_class_lines));
    diags.extend(check_nesting_depth(&lines, source, config.max_nesting_depth));

    diags
}
```

### 2. fn line count check

```rust
fn check_function_lengths(
    lines: &[&str],
    source: &SourceFile,
    max_lines: usize,
) -> Vec<Diagnostic> {
    // AST parse → find all FunctionDef/AsyncFunctionDef
    // Get range (start_line..end_line)
    // range.len() > max_lines → emit diagnostic
    let ast = parse_ast(&source.content)?;
    let mut diags = Vec::new();
}
```

### 3. nesting depth check

```rust
fn check_nesting_depth(
    lines: &[&str],
    source: &SourceFile,
    max_depth: usize,
) -> Vec<Diagnostic> {
    // Recurse AST, track depth
    // Enter If/For/While/With/Try/FunctionDef → depth + 1
    // depth > max_depth → emit diagnostic
}
```

## 测试

```rust
#[test]
fn function_too_long() {
    // build fn > 50 lines
}

#[test]
fn class_too_long() {
    // build class > 300 lines
}

#[test]
fn file_too_long() {
    // build file > 1000 lines
}

#[test]
fn nesting_too_deep() {
    let src = concat!(
        "def foo():\n",
        "    if True:\n",
        "        if True:\n",
        "            if True:\n",
        "                if True:\n",
        "                    if True:\n",
        "                        pass\n",
    );
    // depth 5 > max 4 → report
}

#[test]
fn no_warning_within_limits() {
    let src = "def foo():\n    pass\n";
    // no warning expected
}
```

## 验证

```bash
cargo test -- complexity
```
