# Task 17: Checker — magic_number (AST-based)

> Status: ⬜ Not started
> Depends: Task 05
> Output: Detect unnamed numeric literals in code

## Goal

AST-based magic number detection:
- Identify numeric literals in expressions (not assignment LHS / const defs)
- Configurable allow-list (default: 0, 1, -1, 2)
- Ignore: `range()`, slice, `__version__`, type annotations, default params

## Reference

- `cclinter/src/checker/magic_number.rs` — structure reference

## Implementation

### 1. checker/magic_number.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::MagicNumberConfig;
use std::collections::HashSet;

pub fn check_magic_number(
    source: &SourceFile,
    config: &MagicNumberConfig,
) -> Vec<Diagnostic> {
    if !config.enabled {
        return vec![];
    }
    let allowed: HashSet<i64> = config.allowed.iter().copied().collect();
    let lines = source.lines();
    let mut diags = Vec::new();

    // AST walk, find Constant (numeric) nodes
    let ast = parse_ast(&source.content)?;

    walk_for_magic_numbers(&ast, &lines, source, &allowed, &mut diags);

    diags
}
```

### 2. AST walk

```rust
fn walk_for_magic_numbers(
    ast: &Mod,
    lines: &[&str],
    source: &SourceFile,
    allowed: &HashSet<i64>,
    diags: &mut Vec<Diagnostic>,
) {
    // Walk all Constant::Int / Constant::Float in expressions
    // Exclude:
    //   - Const defs on assignment LHS (MY_CONST = 42)
    //   - Default params (def foo(x=1):)
    //   - range() call args
    //   - Slice expressions
    //   - Type annotations (Literal[1])
    //   - __version__ assignment
    // Remaining numbers: check against allowed set
    // Not in set → emit diagnostic
}
```

## Tests

```rust
#[test]
fn detect_magic_number() {
    let src = "x = calculate(42)\n";
    // 42 not in allowed [0,1,-1,2] → report
}

#[test]
fn allowed_numbers_no_warning() {
    let src = "x = y + 1\nz = w - 0\n";
    // 1 and 0 in allowed → no report
}

#[test]
fn constant_definition_no_warning() {
    let src = "MAX_SIZE = 100\n";
    // 100 in const def → no report
}

#[test]
fn default_param_no_warning() {
    let src = "def foo(timeout=30):\n    pass\n";
    // 30 in default param → no report
}

#[test]
fn range_no_warning() {
    let src = "for i in range(10):\n    pass\n";
    // 10 in range() → no report
}
```

## Verify

```bash
cargo test -- magic_number
```
