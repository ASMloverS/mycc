# Task 19: Checker — Prohibited 禁止函数/模块 (AST-based)

> Status: ⬜ Not started
> Deps: Task 05
> Output: Detect prohibited fn/module calls

## Goal

AST-based detection of prohibited fn calls and module usage:
- Default blacklist: `eval`, `exec`, `__import__`, `os.system`, `subprocess.call` (without shell=False)
- Config can add/remove entries

## Reference

- `cclinter/src/checker/prohibited.rs`

## Steps

### 1. checker/prohibited.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::ProhibitedConfig;

fn default_prohibited() -> Vec<String> {
    vec![
        "eval".into(),
        "exec".into(),
        "__import__".into(),
        "os.system".into(),
    ]
}

pub fn check_prohibited(
    source: &SourceFile,
    config: &ProhibitedConfig,
) -> Vec<Diagnostic> {
    let lines = source.lines();
    let ast = parse_ast(&source.content)?;

    let mut prohibited = Vec::new();
    if config.use_default {
        prohibited.extend(default_prohibited());
    }
    prohibited.extend(config.extra.iter().cloned());
    for remove in &config.remove {
        prohibited.retain(|p| p != remove);
    }

    let mut diags = Vec::new();

    // Walk AST Call nodes
    walk_for_prohibited_calls(&ast, &lines, source, &prohibited, &mut diags);

    diags
}
```

### 2. AST Walk

```rust
fn walk_for_prohibited_calls(
    ast: &Mod,
    lines: &[&str],
    source: &SourceFile,
    prohibited: &[String],
    diags: &mut Vec<Diagnostic>,
) {
    // Walk all Call nodes
    // Get full call path:
    //   Name("eval") → "eval"
    //   Attribute(Name("os"), "system") → "os.system"
    //   Attribute(Attribute(Name("subprocess"), "call"), ...) → "subprocess.call"
    // Match against prohibited list
    // Match → emit diagnostic
}

fn get_call_path(call: &Expr) -> String {
    // Recursively build call path: "os.system", "eval", etc
}
```

## Tests

```rust
#[test]
fn detect_eval() {
    let src = "x = eval('1+1')\n";
    // Must report prohibited eval
}

#[test]
fn detect_os_system() {
    let src = "import os\nos.system('ls')\n";
    // Must report prohibited os.system
}

#[test]
fn no_warning_for_allowed() {
    let src = "x = int('42')\n";
    // int not in blacklist → no report
}

#[test]
fn custom_prohibited() {
    let config = ProhibitedConfig {
        use_default: false,
        extra: vec!["print".into()],
        remove: vec![],
    };
    let src = "print('hello')\n";
    // Must report prohibited print
}

#[test]
fn remove_default_prohibited() {
    let config = ProhibitedConfig {
        use_default: true,
        extra: vec![],
        remove: vec!["eval".into()],
    };
    let src = "x = eval('1+1')\n";
    // eval removed from list → no report
}
```

## Verify

```bash
cargo test -- prohibited
```
