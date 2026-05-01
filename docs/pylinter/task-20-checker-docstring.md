# Task 20: Checker — docstring (AST-based)

> Status: ⬜ Not started
> Depends: Task 05
> Output: Check module/class/fn docstring presence

## Goal

AST-based docstring checks. All toggleable:
- Module docstring
- Class docstring
- Fn/method docstring

## Steps

### 1. checker/docstring.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::DocstringConfig;

pub fn check_docstring(
    source: &SourceFile,
    config: &DocstringConfig,
) -> Vec<Diagnostic> {
    if !config.module && !config.class && !config.function {
        return vec![];
    }
    let lines = source.lines();
    let ast = parse_ast(&source.content)?;

    let mut diags = Vec::new();

    // 1. Module docstring
    if config.module {
        if !has_module_docstring(&ast) {
            diags.push(Diagnostic::new(
                source.display_path(),
                1, 1,
                Severity::Note,
                "readability-missing-module-docstring",
                "Module is missing a docstring",
            ));
        }
    }

    // 2. Class docstring
    if config.class {
        diags.extend(check_class_docstrings(&ast, &lines, source));
    }

    // 3. Function docstring
    if config.function {
        diags.extend(check_function_docstrings(&ast, &lines, source));
    }

    diags
}
```

### 2. Docstring detection

```rust
fn has_module_docstring(ast: &Mod) -> bool {
    // Module docstring = AST body first stmt is Expr(Constant(string))
    // body[0] is Expr { value: Constant { value: String("...") } }
}

fn has_docstring(body: &[Statement]) -> bool {
    if body.is_empty() {
        return false;
    }
    matches!(&body[0], Statement::Expr { value: Expr::Constant(Constant::Str(_)) })
}

fn check_class_docstrings(
    ast: &Mod,
    lines: &[&str],
    source: &SourceFile,
) -> Vec<Diagnostic> {
    // Walk AST → find ClassDef
    // Check body[0] has docstring
    // Missing → emit diagnostic
}

fn check_function_docstrings(
    ast: &Mod,
    lines: &[&str],
    source: &SourceFile,
) -> Vec<Diagnostic> {
    // Walk AST → find all FunctionDef/AsyncFunctionDef
    // Check body[0] has docstring
    // Missing → emit diagnostic
    // Exclude: __init__ (config-dependent)
    // Exclude: one-line lambda, simple getter/setter
}
```

## Tests

```rust
#[test]
fn module_with_docstring_no_warning() {
    let src = "\"\"\"Module docstring.\"\"\"\n\nx = 1\n";
    // No report
}

#[test]
fn module_without_docstring() {
    let src = "x = 1\n";
    // Report missing-module-docstring
}

#[test]
fn class_with_docstring_no_warning() {
    let src = "class Foo:\n    \"\"\"Class doc.\"\"\"\n    pass\n";
    // No report
}

#[test]
fn class_without_docstring() {
    let src = "class Foo:\n    pass\n";
    // Report missing-class-docstring
}

#[test]
fn function_with_docstring_no_warning() {
    let src = "def foo():\n    \"\"\"Function doc.\"\"\"\n    pass\n";
    // No report
}

#[test]
fn function_without_docstring() {
    let src = "def foo():\n    pass\n";
    // Report missing-function-docstring
}

#[test]
fn disabled_check() {
    let config = DocstringConfig { module: false, class: false, function: false };
    let src = "x = 1\n";
    // No report
}
```

## Verify

```bash
cargo test -- docstring
```
