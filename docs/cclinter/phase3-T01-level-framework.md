### Task 01: Analysis Level Framework

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/mod.rs`
- Test: `tests/analyzer_tests.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/analyzer_tests.rs`:

```rust
use cclinter::analyzer::analyze_source;
use cclinter::common::source::SourceFile;
use cclinter::config::Config;
use std::path::PathBuf;

#[test]
fn test_none_level_no_diags() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = analyze_source(&src, "none");
    assert!(diags.is_empty());
}

#[test]
fn test_basic_level_runs() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = analyze_source(&src, "basic");
    assert!(diags.len() >= 0);
}

#[test]
fn test_invalid_level_defaults_none() {
    let input = "int x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = analyze_source(&src, "invalid");
    assert!(diags.is_empty());
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests`
Expected: FAIL.

- [ ] **Step 3: Implement `src/analyzer/mod.rs`**

```rust
pub mod basic;
pub mod strict;
pub mod deep;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn analyze_source(source: &SourceFile, level: &str) -> Vec<Diagnostic> {
    match level {
        "basic" => basic::check(source),
        "strict" => {
            let mut d = basic::check(source);
            d.extend(strict::check(source));
            d
        }
        "deep" => {
            let mut d = basic::check(source);
            d.extend(strict::check(source));
            d.extend(deep::check(source));
            d
        }
        _ => vec![],
    }
}
```

Create `src/analyzer/basic.rs`:

```rust
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn check(_source: &SourceFile) -> Vec<Diagnostic> {
    vec![]
}
```

Create `src/analyzer/strict.rs`:

```rust
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn check(_source: &SourceFile) -> Vec<Diagnostic> {
    vec![]
}
```

Create `src/analyzer/deep.rs`:

```rust
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn check(_source: &SourceFile) -> Vec<Diagnostic> {
    vec![]
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "🚧 feat(cclinter): analysis level framework (basic/strict/deep)"
```
