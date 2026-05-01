# Task 25: Analyzer Pipeline Integration + CLI Wiring + E2E Tests

> Status: ⬜ Not started
> Depends on: Task 21, Task 22, Task 23, Task 24
> Output: Complete format + check + analyze pipeline, all features E2E usable

## Goal

1. Build analyzer pipeline (basic/strict/deep tiers)
2. Wire analyzer in cli.rs, exit code logic matching cclinter
3. Write full E2E tests

## Reference

- `cclinter/src/analyzer/mod.rs` — 3-tier dispatch
- `cclinter/src/cli.rs` — analyzer invocation logic

## Steps

### 1. analyzer/mod.rs

```rust
pub mod basic;
pub mod deep;
pub mod strict;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::{AnalysisConfig, AnalysisLevel};

pub fn analyze_source(
    source: &SourceFile,
    level: &AnalysisLevel,
    config: &AnalysisConfig,
) -> Vec<Diagnostic> {
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

### 2. cli.rs — Analyzer Wiring

Follow cclinter cli.rs pattern:

```rust
if !args.format_only && config.analysis.level != AnalysisLevel::None {
    let analysis_config = &config.analysis;
    let analysis_level = &analysis_config.level;
    let all_analysis_diags: Vec<Diagnostic> = files
        .par_iter()
        .flat_map(|file_path| {
            let source = SourceFile::load(file_path).unwrap_or_else(...);
            analyzer::analyze_source(&source, analysis_level, analysis_config)
        })
        .collect();
    // Output diags, exit_code |= 4
}
```

### 3. E2E Tests

**tests/fixtures/input/full_analysis.py**:
```python
import os
import unused_module
from typing import TYPE_CHECKING

# Mutable default
def foo(items=[]):
    eval("1+1")
    x = 1
    return 2

# Missing docstring
class Bar:
    # Shadow builtin
    def process(list):
        if list == None:
            return True
        else:
            return False

# Bare except
try:
    foo()
except:
    pass
```

```rust
// tests/analyzer_tests.rs

#[test]
fn basic_level_catches_mutable_default() { ... }
#[test]
fn basic_level_catches_bare_except() { ... }
#[test]
fn strict_level_catches_unnecessary_pass() { ... }
#[test]
fn deep_level_catches_unused_variable() { ... }
#[test]
fn deep_level_catches_shadow_builtin() { ... }

#[test]
fn level_none_no_diagnostics() {
    // analysis_level = None → no analyzer output
}

#[test]
fn full_pipeline_end_to_end() {
    // format + check + analyze full run
    // Verify exit code + diagnostic output
}
```

### 4. Integration Tests

```rust
// tests/integration_tests.rs

#[test]
fn idempotent_full_pipeline() {
    // Run full pipeline twice → same result
}

#[test]
fn check_mode_returns_nonzero() {
    // Problematic file → exit code != 0
}

#[test]
fn clean_file_returns_zero() {
    // Fully compliant file → exit code = 0
}
```

## Verify

```bash
cargo test
cargo run -- --check --verbose tests/fixtures/input/full_analysis.py
cargo run -- --diff tests/fixtures/input/full_analysis.py
cargo run -- -i tests/fixtures/input/full_analysis.py
```
