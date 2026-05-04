# Task 11: Formatter — import_sort (PEP 8 / isort)

> Status: ✅ Done
> Depends: Task 05, Task 10
> Output: imports sorted stdlib → third-party → local

## Goal

isort-style import sorting:
- Groups: stdlib → third-party → local
- Alphabetical within each group
- 1 blank line between groups
- Merge same-module `from` imports

## PEP 8 Import Rules

1. Imports top of file, after module docstring/comments
2. Three groups, blank-line separated: stdlib / third-party / local
3. Alphabetical within each group

## Implementation

### 1. formatter/import_sort.rs

```rust
use crate::common::source::SourceFile;
use crate::config::FormatConfig;
use crate::cst::tokens::tokenize_source;

#[derive(Debug, Clone)]
enum ImportKind {
    Import { module: String },
    FromImport { module: String, names: Vec<String> },
}

#[derive(Debug, Clone)]
struct ImportEntry {
    kind: ImportKind,
    line_num: usize,
    indent: String,       // preserve indent (may be inside if TYPE_CHECKING:)
    original_line: String,
}

pub fn fix_import_sort(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.import_sorting == ImportSorting::Disabled {
        return Ok(());
    }

    // 1. Parse AST, find all import stmts
    let ast = parse_ast(&source.content)?;

    // 2. Find top-level import block (consecutive imports, possibly comment/blank-line separated)
    let import_block = find_import_block(&source.content, &ast)?;

    if import_block.is_empty() {
        return Ok(());
    }

    // 3. Group
    let groups = group_imports(&import_block);

    // 4. Sort within groups
    let sorted = sort_groups(&groups);

    // 5. Replace import block in source
    replace_import_block(source, &import_block, &sorted);

    Ok(())
}
```

### 2. Grouping Logic

```rust
fn classify_import(module: &str) -> ImportGroup {
    // Hardcode common stdlib, or use known list
    let stdlib = ["os", "sys", "re", "json", "collections", ...];
    // First segment in stdlib → Stdlib
    // Starts with '.' → Local (relative import)
    // Else → ThirdParty
}

enum ImportGroup { Stdlib, ThirdParty, Local }
```

> Exact stdlib vs third-party hard without package manager info. Simplified approach:
> - Maintain Python 3.12 stdlib module list
> - `.` prefix → relative import → Local
> - Rest → ThirdParty

### 3. Merge from Imports

```python
# Before:
from os import path
from os import environ

# After:
from os import environ, path
```

## Tests

```rust
#[test]
fn sort_stdlib_imports() {
    let input = "import sys\nimport os\n";
    let expected = "import os\nimport sys\n";
}

#[test]
fn group_stdlib_and_third_party() {
    let input = "import requests\nimport os\nimport sys\n";
    let expected = "import os\nimport sys\n\nimport requests\n";
}

#[test]
fn sort_from_imports() {
    let input = "from os import path\nfrom os import environ\n";
    let expected = "from os import environ, path\n";
}

#[test]
fn preserve_indent_in_type_checking_block() {
    let input = "from typing import TYPE_CHECKING\n\nif TYPE_CHECKING:\n    import os\n    import sys\n";
    // imports inside if TYPE_CHECKING must NOT move to top level
}

#[test]
fn no_imports_no_change() {
    let input = "x = 1\n";
    // unchanged
}
```

## Verify

```bash
cargo test -- import_sort
```
