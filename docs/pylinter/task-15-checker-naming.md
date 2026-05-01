# Task 15: Checker — naming 命名检查 (AST-based)

> 状态: ⬜ 未开始
> 依赖: Task 05
> 产出: configurable PEP 8 naming check → Diagnostic

## Goal

AST-based Python identifier naming check:
- fn/method: `snake_case` default
- class: `pascal_case` default
- constant: `upper_snake_case` default
- variable: `snake_case` default
- module: `snake_case` default

All rules configurable. Match cclinter naming checker style.

## Ref

- `cclinter/src/checker/naming.rs` — regex match + naming style check

## Steps

### 1. checker/naming.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::NamingConfig;
use regex::Regex;
use std::sync::LazyLock;

static SNAKE_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap());
static UPPER_SNAKE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z_][A-Z0-9_]*$").unwrap());
static PASCAL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap());
static CAMEL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap());

pub fn check_naming(source: &SourceFile, config: &NamingConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // AST → precise identifier positions
    let ast = parse_ast(&source.content)?;
    let lines = source.lines();

    // Walk AST:
    // FunctionDef / AsyncFunctionDef → check fn naming
    // ClassDef → check class naming
    // Assign uppercase-leading → check constant naming
    // Assign other → check variable naming
    // Module name → infer from file path

    walk_ast(&ast, &lines, source, config, &mut diags);

    diags
}
```

### 2. AST Walk

```rust
fn walk_ast(
    ast: &Mod,
    lines: &[&str],
    source: &SourceFile,
    config: &NamingConfig,
    diags: &mut Vec<Diagnostic>,
) {
    // Walk ast.body:
    //   match node {
    //     Statement::FunctionDef { name, .. } =>
    //       check_name(name, &config.function, "function", line, source, diags)
    //     Statement::ClassDef { name, body, .. } => {
    //       check_name(name, &config.class, "class", line, source, diags)
    //       walk body methods
    //     }
    //     Statement::Assign { targets, .. } =>
    //       uppercase / UPPER_SNAKE → constant check
    //       else → variable check
    //     ...
    //   }
}

fn check_name(
    name: &str,
    style: &NamingStyle,
    kind: &str,
    line_num: usize,
    source: &SourceFile,
    diags: &mut Vec<Diagnostic>,
) {
    let re = naming_regex(style);
    if !re.is_match(name) {
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            line_num,
            1,
            Severity::Warning,
            &format!("readability-naming-{}", kind),
            &format!("{} '{}' does not follow {} convention", kind, name, style.as_str()),
            lines[line_num - 1],
        ));
    }
}
```

## Tests

```rust
#[test]
fn function_snake_case() {
    let src = "def MyFunction():\n    pass\n";
    // expect naming-function warning
}

#[test]
fn class_pascal_case() {
    let src = "class my_class:\n    pass\n";
    // expect naming-class warning
}

#[test]
fn constant_upper_snake() {
    let src = "myConstant = 42\n";
    // expect naming-constant warning (if detected constant)
}

#[test]
fn no_warning_correct_naming() {
    let src = "def my_function():\n    my_var = 1\n";
    // expect no warning
}
```

## Verify

```bash
cargo test -- naming
```
