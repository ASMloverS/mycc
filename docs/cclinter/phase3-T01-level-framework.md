### Task 01: Analysis Level Framework

**Files:**
- Modify: `tools/linter/cclinter/src/analyzer/mod.rs`
- Test: `tests/analyzer_tests.rs`

- [x] **Step 1: Write failing tests**

Create `tests/analyzer_tests.rs`:

```rust
use cclinter::analyzer::analyze_source;
use cclinter::common::source::SourceFile;
use cclinter::config::{AnalysisConfig, AnalysisLevel};
use std::path::PathBuf;

#[test]
fn test_none_level_no_diags() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = AnalysisConfig::default();
    let diags = analyze_source(&src, &AnalysisLevel::None, &config);
    assert!(diags.is_empty());
}

#[test]
fn test_basic_level_runs() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = AnalysisConfig::default();
    let _ = analyze_source(&src, &AnalysisLevel::Basic, &config);
}

#[test]
fn test_empty_source() {
    let input = "";
    let src = SourceFile::from_string(input, PathBuf::from("empty.c"));
    let config = AnalysisConfig::default();
    let diags = analyze_source(&src, &AnalysisLevel::Deep, &config);
    assert!(diags.is_empty());
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test analyzer_tests`
Expected: FAIL.

- [x] **Step 3: Implement `src/analyzer/mod.rs`**

```rust
pub mod basic;
pub mod strict;
pub mod deep;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::{AnalysisConfig, AnalysisLevel};

pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel, config: &AnalysisConfig) -> Vec<Diagnostic> {
    match level {
        AnalysisLevel::None => vec![],
        AnalysisLevel::Basic => basic::check(source, config),
        AnalysisLevel::Strict => {
            let mut diags = basic::check(source, config);
            diags.extend(strict::check(source, config));
            diags
        }
        AnalysisLevel::Deep => {
            let mut diags = basic::check(source, config);
            diags.extend(strict::check(source, config));
            diags.extend(deep::check(source, config));
            diags
        }
    }
}
```

Create `src/analyzer/basic.rs`:

```rust
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;

pub fn check(_source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    vec![]
}
```

Create `src/analyzer/strict.rs`:

```rust
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;

pub fn check(_source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    vec![]
}
```

Create `src/analyzer/deep.rs`:

```rust
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;

pub fn check(_source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    vec![]
}
```

- [x] **Step 4: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "🚧 feat(cclinter): analysis level framework (basic/strict/deep)"
```
