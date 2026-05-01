# Task 24: Analyzer — deep Level

> Status: ⬜ Not started
> Deps: Task 05
> Output: deep rules — unreachable code, unused vars, shadow builtins

## Goal

3 rules for analyzer deep level:

| Rule | rule_id | Severity |
|---|---|---|
| Unreachable code | `deadcode-unreachable` | Warning |
| Unused variable | `deadcode-unused-variable` | Warning |
| Shadow builtin | `bugprone-shadow-builtin` | Warning |

## Ref

- `cclinter/src/analyzer/deep.rs` — structure ref

## Steps

### 1. analyzer/deep.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_unreachable_code(source));
    diags.extend(check_unused_variables(source));
    diags.extend(check_shadow_builtin(source));
    diags
}
```

### 2. Unreachable Code

```rust
fn check_unreachable_code(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk stmt sequence inside fn body
    // After return/break/continue/raise → report remaining stmts
    // Exclude: pass, docstring, ...
}
```

```python
def foo():
    return 1
    x = 2  # Warning: unreachable code
```

### 3. Unused Variables

```rust
fn check_unused_variables(source: &SourceFile) -> Vec<Diagnostic> {
    // Per fn scope:
    // 1. Collect assigned names (Assign, AugAssign, For target, ...)
    // 2. Collect referenced names (Name with Load context)
    // 3. Assigned but never referenced → report
    // Exclude: _ prefix (convention: unused)
    // Exclude: global/nonlocal declared
}
```

### 4. Shadow Builtins

```rust
fn check_shadow_builtin(source: &SourceFile) -> Vec<Diagnostic> {
    let builtins = [
        "list", "dict", "set", "tuple", "str", "int", "float", "bool",
        "type", "id", "input", "print", "len", "range", "enumerate",
        "zip", "map", "filter", "sorted", "reversed", "sum", "min", "max",
        "abs", "round", "any", "all", "open", "hash", "dir", "vars",
        "super", "property", "staticmethod", "classmethod",
        "object", "Exception", "BaseException",
    ];

    // Walk fn params, assign targets, for-loop targets
    // Name matches builtin → report
}
```

## Tests

```rust
#[test]
fn unreachable_after_return() {
    let src = "def foo():\n    return 1\n    x = 2\n";
    // Expect deadcode-unreachable
}

#[test]
fn reachable_code_ok() {
    let src = "def foo():\n    x = 1\n    return x\n";
    // No report
}

#[test]
fn unused_variable() {
    let src = "def foo():\n    x = 1\n    return 2\n";
    // Expect deadcode-unused-variable: x
}

#[test]
fn used_variable_ok() {
    let src = "def foo():\n    x = 1\n    return x\n";
    // No report
}

#[test]
fn underscore_prefix_ok() {
    let src = "def foo():\n    _unused = 1\n    return 2\n";
    // No report (_ prefix = unused convention)
}

#[test]
fn shadow_builtin_list() {
    let src = "def foo(list):\n    pass\n";
    // Expect bugprone-shadow-builtin: list
}

#[test]
fn normal_name_ok() {
    let src = "def foo(items):\n    pass\n";
    // No report
}
```

## Verify

```bash
cargo test -- analyzer::deep
```
