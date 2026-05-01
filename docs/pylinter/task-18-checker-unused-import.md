# Task 18: Checker — unused_import (AST-based)

> Status: ⬜ Not started
> Depends: Task 05
> Output: Detect imported-but-unused modules/names

## Goal

AST-based unused import detection:
- `import os` → check `os` referenced later
- `from os import path` → check `path` referenced
- Exclude: `__init__.py` re-exports, `TYPE_CHECKING` block imports, conditional `typing.TYPE_CHECKING` imports

## Reference

- `cclinter/src/checker/unused.rs` — pattern ref (declare vs use)

## Steps

### 1. checker/unused_import.rs

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::UnusedImportConfig;
use std::collections::{HashMap, HashSet};

struct ImportInfo {
    name: String,
    line: usize,
    is_from: bool,
    module: String,
}

pub fn check_unused_import(
    source: &SourceFile,
    config: &UnusedImportConfig,
) -> Vec<Diagnostic> {
    if !config.enabled {
        return vec![];
    }
    let lines = source.lines();
    let ast = parse_ast(&source.content)?;

    // 1. Collect all import stmts
    let imports = collect_imports(&ast);

    // 2. Collect all name refs (Name, Attribute nodes)
    let used_names = collect_used_names(&ast);

    // 3. Find unused imports
    let mut diags = Vec::new();
    for imp in &imports {
        if !used_names.contains(&imp.name) {
            diags.push(Diagnostic::new_with_source(
                source.display_path(),
                imp.line,
                1,
                Severity::Warning,
                "readability-unused-import",
                &format!("'{}' is imported but unused", imp.name),
                lines[imp.line - 1],
            ));
        }
    }
    diags
}
```

### 2. Collect imports

```rust
fn collect_imports(ast: &Mod) -> Vec<ImportInfo> {
    // Walk AST top-level stmts:
    //   Import { names: [Alias { name, asname }] }
    //     → has asname? use asname : last segment of name
    //   ImportFrom { module, names: [Alias { name, asname }] }
    //     → has asname? use asname : use name
    // Exclude: imports inside `if TYPE_CHECKING:` block
}
```

### 3. Collect usage

```rust
fn collect_used_names(ast: &Mod) -> HashSet<String> {
    // Walk all Name nodes (Load context)
    // Walk all Attribute node values (if value is Name)
    // Collect all referenced names
}
```

## Tests

```rust
#[test]
fn unused_import() {
    let src = "import os\nx = 1\n";
    // os unused → report
}

#[test]
fn used_import_no_warning() {
    let src = "import os\nprint(os.path)\n";
    // os used → no report
}

#[test]
fn unused_from_import() {
    let src = "from os import path\nx = 1\n";
    // path unused → report
}

#[test]
fn aliased_import() {
    let src = "import numpy as np\nprint(np.array)\n";
    // np used → no report
}

#[test]
fn type_checking_import_excluded() {
    let src = "from typing import TYPE_CHECKING\n\nif TYPE_CHECKING:\n    import os\n";
    // TYPE_CHECKING block import → exclude from report
}
```

## Verify

```bash
cargo test -- unused_import
```
