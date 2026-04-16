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

- [ ] **Step 4: Create `src/cli.rs` — clap arg definitions + run()**

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cclinter", version, about = "C language linter")]
pub struct Args {
    #[arg(required = true)]
    pub paths: Vec<std::path::PathBuf>,

    #[arg(long)]
    pub config: Option<std::path::PathBuf>,

    #[arg(short, long)]
    pub in_place: bool,

    #[arg(long)]
    pub check: bool,

    #[arg(long)]
    pub diff: bool,

    #[arg(long)]
    pub format_only: bool,

    #[arg(long, value_enum, default_value = "none")]
    pub analysis_level: AnalysisLevel,

    #[arg(short, long)]
    pub jobs: Option<usize>,

    #[arg(long)]
    pub exclude: Vec<String>,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short = 'v', long)]
    pub verbose: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, Default)]
pub enum AnalysisLevel {
    #[default]
    None,
    Basic,
    Strict,
    Deep,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _args = Args::parse();
    Ok(())
}
```

- [ ] **Step 5: Create `src/common/mod.rs`, `diag.rs`, `source.rs`, `rule.rs` — stubs**

```rust
// src/common/mod.rs
pub mod diag;
pub mod source;
pub mod rule;
```

```rust
// src/common/source.rs
use std::path::PathBuf;

pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub lines: Vec<String>,
}

impl SourceFile {
    pub fn from_string(content: &str, path: PathBuf) -> Self {
        Self {
            path,
            content: content.to_string(),
            lines: content.lines().map(|l| l.to_string()).collect(),
        }
    }
}
```

```rust
// src/common/diag.rs
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
}

pub enum Severity {
    Warning,
    Error,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sev = match self.severity {
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(
            f,
            "{}:{}:{}: {}: {} [{}]",
            self.file, self.line, self.col, sev, self.message, self.rule_id
        )
    }
}
```

```rust
// src/common/rule.rs
pub trait Rule {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
}
```

- [ ] **Step 6: Create `src/formatter/mod.rs`, `src/checker/mod.rs`, `src/analyzer/mod.rs` — stubs**

```rust
// src/formatter/mod.rs
use crate::common::source::SourceFile;

pub fn format_source(source: &SourceFile) -> SourceFile {
    source.clone()
}
```

```rust
// src/checker/mod.rs
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn check_source(_source: &SourceFile) -> Vec<Diagnostic> {
    vec![]
}
```

```rust
// src/analyzer/mod.rs
use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn analyze_source(_source: &SourceFile) -> Vec<Diagnostic> {
    vec![]
}
```

- [ ] **Step 7: Create `src/config.rs` — stub**

```rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Deserialize)]
pub struct FormatConfig {
    pub column_limit: Option<usize>,
    pub indent_width: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CheckConfig {}

#[derive(Debug, Deserialize)]
pub struct AnalysisConfig {
    pub level: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: FormatConfig {
                column_limit: Some(120),
                indent_width: Some(2),
            },
            check: CheckConfig {},
            analysis: AnalysisConfig { level: None },
        }
    }
}

pub fn load_config(path: Option<&PathBuf>) -> Config {
    match path {
        Some(p) if p.exists() => {
            let content = std::fs::read_to_string(p).unwrap_or_default();
            serde_yaml::from_str(&content).unwrap_or_default()
        }
        _ => Config::default(),
    }
}
```

- [ ] **Step 8: Create `src/ignore.rs` — stub**

```rust
pub struct IgnoreMatcher {
    patterns: Vec<globset::GlobMatcher>,
}

impl IgnoreMatcher {
    pub fn from_file(_path: &std::path::Path) -> Self {
        Self { patterns: vec![] }
    }

    pub fn is_ignored(&self, _path: &std::path::Path) -> bool {
        false
    }
}
```

- [ ] **Step 9: Add `Clone` derive to `SourceFile`**

Add `#[derive(Clone)]` to `SourceFile` struct in `src/common/source.rs`.

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
