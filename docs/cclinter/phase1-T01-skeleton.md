### Task 01: Project Skeleton

**Files:**
- Create: `tools/linter/cclinter/Cargo.toml`
- Create: `tools/linter/cclinter/src/main.rs`
- Create: `tools/linter/cclinter/src/lib.rs`
- Create: `tools/linter/cclinter/src/cli.rs`
- Create: `tools/linter/cclinter/src/config.rs`
- Create: `tools/linter/cclinter/src/ignore.rs`
- Create: `tools/linter/cclinter/src/common/mod.rs`
- Create: `tools/linter/cclinter/src/common/diag.rs`
- Create: `tools/linter/cclinter/src/common/source.rs`
- Create: `tools/linter/cclinter/src/common/rule.rs`
- Create: `tools/linter/cclinter/src/formatter/mod.rs`
- Create: `tools/linter/cclinter/src/checker/mod.rs`
- Create: `tools/linter/cclinter/src/analyzer/mod.rs`
- Test: `cargo build` in `tools/linter/cclinter/`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "cclinter"
version = "0.1.0"
edition = "2021"
description = "C language linter: format + style check + static analysis"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
regex = "1"
rayon = "1"
walkdir = "2"
globset = "0.4"
colored = "2"
similar = "2"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Create `src/lib.rs` — module declarations**

```rust
pub mod cli;
pub mod common;
pub mod config;
pub mod formatter;
pub mod checker;
pub mod analyzer;
pub mod ignore;
```

- [ ] **Step 3: Create `src/main.rs` — entry point**

```rust
use cclinter::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("{}", e);
        std::process::exit(8);
    }
}
```

- [x] **Step 4: Create `src/cli.rs` — clap arg definitions + run()**

```rust
use clap::Parser;
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use crate::config::{load_config, AnalysisLevel};

#[derive(Parser, Debug)]
#[command(name = "cclinter", version, about = "C language linter")]
pub struct Args {
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(short, long, conflicts_with = "check", conflicts_with = "diff")]
    pub in_place: bool,

    #[arg(long, conflicts_with = "diff", conflicts_with = "in_place")]
    pub check: bool,

    #[arg(long, conflicts_with = "check", conflicts_with = "in_place")]
    pub diff: bool,

    #[arg(long)]
    pub format_only: bool,

    #[arg(long, value_enum)]
    pub analysis_level: Option<AnalysisLevel>,

    #[arg(short, long, value_parser = parse_jobs)]
    pub jobs: Option<usize>,

    #[arg(long)]
    pub exclude: Vec<String>,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short = 'v', long)]
    pub verbose: bool,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Full implementation with format → check → analyze pipeline
}
```

Note: `AnalysisLevel` is defined in `config.rs`, not `cli.rs`.

- [x] **Step 5: Create `src/common/mod.rs`, `diag.rs`, `source.rs`, `rule.rs` — stubs**

```rust
// src/common/mod.rs
pub mod diag;
pub mod source;
pub mod rule;
pub mod string_utils;
```

```rust
// src/common/source.rs
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub original: String,
}

impl SourceFile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> { ... }
    pub fn from_string(content: &str, path: PathBuf) -> Self { ... }
    pub fn lines(&self) -> Vec<&str> { ... }
    pub fn line_count(&self) -> usize { ... }
    pub fn is_modified(&self) -> bool { ... }
}

pub fn mask_string_literals(line: &str) -> String { ... }
pub fn strip_line_comment(line: &str) -> &str { ... }
```

```rust
// src/common/diag.rs
use colored::Colorize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Severity {
    Note,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
    pub source_line: Option<String>,
}

impl Diagnostic {
    pub fn new(...) -> Self { ... }
    pub fn new_with_source(...) -> Self { ... }
}

impl std::fmt::Display for Diagnostic { ... }  // colored clang-tidy format
```

```rust
// src/common/rule.rs
pub trait Rule {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn severity(&self) -> Severity;
    fn check(&self, source: &SourceFile) -> Vec<Diagnostic>;
}
```

```rust
// src/common/string_utils.rs
pub fn split_outside_strings(s: &str) -> Vec<String> { ... }
```

- [x] **Step 6: Create `src/formatter/mod.rs`, `src/checker/mod.rs`, `src/analyzer/mod.rs` — stubs**

```rust
// src/formatter/mod.rs
pub fn format_source(source: &mut SourceFile, config: &FormatConfig) -> Result<Vec<Diagnostic>, Box<dyn Error>>;
```

```rust
// src/checker/mod.rs
pub fn check_source(source: &SourceFile, config: &CheckConfig) -> Vec<Diagnostic>;
```

```rust
// src/analyzer/mod.rs
pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel, config: &AnalysisConfig) -> Vec<Diagnostic>;
```

- [x] **Step 7: Create `src/config.rs` — full config with serde**

All config structs use `#[serde(default)]` with non-Optional fields and explicit `Default` impls. Enums: `AnalysisLevel`, `PointerAlignment`, `BraceStyle`, `IncludeSorting`, `CommentStyle`, `LineEnding`, `NamingStyle`, `IncludeGuardStyle`.

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FormatConfig {
    pub column_limit: usize,           // default 120
    pub indent_width: usize,           // default 2
    pub use_tabs: bool,                // default false
    pub pointer_alignment: PointerAlignment,
    pub brace_style: BraceStyle,
    pub switch_case_indent: bool,
    pub blank_lines_before_function: usize,
    pub blank_lines_after_include: usize,
    pub max_consecutive_blank_lines: usize,
    pub space_before_paren: bool,
    pub spaces_around_operators: bool,
    pub include_sorting: IncludeSorting,
    pub comment_style: CommentStyle,
    pub line_ending: LineEnding,
    pub encoding: String,
}

// CheckConfig contains: naming, complexity, magic_number, unused, include_guard, prohibited_functions
// AnalysisConfig contains: level (AnalysisLevel, default Basic)

pub fn find_config() -> Result<Option<PathBuf>, Box<dyn Error>> { ... }
pub fn load_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn Error>> { ... }
```

Config search: `--config` → CWD `.cclinter.yaml` → walk parents → tool binary dir → built-in defaults.

- [x] **Step 8: Create `src/ignore.rs` — gitignore-style pattern matcher**

```rust
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

pub struct IgnoreMatcher {
    set: GlobSet,
}

impl IgnoreMatcher {
    pub fn from_patterns(patterns: &[String]) -> Self { ... }
    pub fn from_string(content: &str) -> Self { ... }
    pub fn from_file(path: &Path) -> Self { ... }
    pub fn is_ignored(&self, path: &Path) -> bool { ... }
    pub fn is_empty(&self) -> bool { ... }
}
```

Pattern expansion: root-only (`/` prefix), directory (`/` suffix), simple names get `**/name/**` + `**/name`. Negation (`!`) not supported.

- [x] **Step 9: `SourceFile` derives `Clone, Debug`, has `original` field for change tracking**

- [ ] **Step 10: Verify build**

Run: `cargo build`
Expected: Compiles with warnings for unused fields at most. No errors.

```bash
cd tools/linter/cclinter && cargo build
```

- [ ] **Step 11: Verify test framework**

Run: `cargo test`
Expected: 0 passed, 0 failed (no tests yet).

```bash
cd tools/linter/cclinter && cargo test
```

- [ ] **Step 12: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "🚧 feat(cclinter): project skeleton with clap CLI and module stubs"
```
