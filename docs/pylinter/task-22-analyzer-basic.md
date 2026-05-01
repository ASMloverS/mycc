# Task 22: Analyzer — basic Level (Common Pitfalls)

> Status: ⬜ Not started
> Depends: Task 05
> Output: basic level analysis rules: mutable default args, missing self, bare except, == None

## Goal

4 rules for analyzer basic level:

| Rule | rule_id | Severity |
|---|---|---|
| Mutable default arg | `bugprone-mutable-default` | Warning |
| Missing self param | `bugprone-missing-self` | Error |
| Bare except | `bugprone-bare-except` | Warning |
| `== None` vs `is None` | `bugprone-none-comparison` | Warning |

## Reference

- `cclinter/src/analyzer/basic.rs` — structure reference

## Steps

### 1. analyzer/basic.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(check_mutable_defaults(source));
    diags.extend(check_missing_self(source));
    diags.extend(check_bare_except(source));
    diags.extend(check_none_comparison(source));
    diags
}
```

### 2. Mutable Default Args

```rust
fn check_mutable_defaults(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk AST FunctionDef/AsyncFunctionDef
    // Check args.defaults + args.kw_defaults
    // Default is List, Dict, Set → report
    // Skip: None, int, str, float, bool, tuple, frozenset
}
```

```python
def foo(items=[]):  # Warning: mutable default
    pass
```

### 3. Missing self

```rust
fn check_missing_self(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk ClassDef → FunctionDef
    // First param not self/cls → report
    // Skip: @staticmethod methods
    // Skip: __init_subclass__, __class_getitem__ specials
}
```

### 4. Bare except

```rust
fn check_bare_except(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk AST ExceptHandler
    // type is None (bare `except:`) → report
}
```

### 5. == None Comparison

```rust
fn check_none_comparison(source: &SourceFile) -> Vec<Diagnostic> {
    // Walk AST Compare nodes
    // ops contains Eq/NotEq + comparators contains None → report
    // Suggest: `is None` / `is not None`
}
```

## Tests

```rust
#[test]
fn mutable_default_list() {
    let src = "def foo(x=[]):\n    pass\n";
    // expect bugprone-mutable-default
}

#[test]
fn mutable_default_dict() {
    let src = "def foo(x={}):\n    pass\n";
    // expect report
}

#[test]
fn immutable_default_ok() {
    let src = "def foo(x=None):\n    pass\n";
    // no report
}

#[test]
fn missing_self() {
    let src = "class Foo:\n    def bar():\n        pass\n";
    // expect bugprone-missing-self
}

#[test]
fn has_self_ok() {
    let src = "class Foo:\n    def bar(self):\n        pass\n";
    // no report
}

#[test]
fn static_method_ok() {
    let src = "class Foo:\n    @staticmethod\n    def bar():\n        pass\n";
    // no report
}

#[test]
fn bare_except() {
    let src = "try:\n    pass\nexcept:\n    pass\n";
    // expect bugprone-bare-except
}

#[test]
fn specific_except_ok() {
    let src = "try:\n    pass\nexcept ValueError:\n    pass\n";
    // no report
}

#[test]
fn none_comparison_eq() {
    let src = "if x == None:\n    pass\n";
    // expect bugprone-none-comparison
}

#[test]
fn none_is_ok() {
    let src = "if x is None:\n    pass\n";
    // no report
}
```

## Verify

```bash
cargo test -- analyzer::basic
```
