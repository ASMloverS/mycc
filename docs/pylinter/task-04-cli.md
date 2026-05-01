# Task 04: CLI + File Collect + Ignore

> Status: ⬜ Not started
> Depends: Task 02, Task 03
> Output: `cargo run -- <path>` discovers .py files, prints formatted results

## Goal

CLI arg parse + file collect + ignore. Match cclinter `cli.rs` + `ignore.rs` style.

## Reference

- `cclinter/src/cli.rs` — Args struct + run() logic
- `cclinter/src/ignore.rs` — IgnoreMatcher

## Steps

### 1. ignore.rs

Copy `IgnoreMatcher` from cclinter. Change: ignore filename → `.pylinterignore`

```rust
pub struct IgnoreMatcher { set: GlobSet }

impl IgnoreMatcher {
    pub fn from_patterns(patterns: &[String]) -> Self { ... }
    pub fn from_string(content: &str) -> Self { ... }
    pub fn from_file(path: &Path) -> Self { ... }
    pub fn is_ignored(&self, path: &Path) -> bool { ... }
    pub fn is_empty(&self) -> bool { ... }
}
```

### 2. cli.rs — Args

```rust
#[derive(Parser, Debug)]
#[command(name = "pylinter", version, about = "Python 3.12+ linter")]
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
```

### 3. cli.rs — run() skeleton

Follow cclinter `run()`. Simplified: format-only pipeline (checker/analyzer placeholder).

```rust
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config(args.config.as_ref())?;
    // ... override analysis_level if specified
    let ignore = build_ignore_matcher(&args);
    let files = collect_files(&args.paths, &ignore)?;
    // ... parallel format (rayon)
    // ... check mode / diff mode / in-place / stdout
    // ... exit codes: 1=format, 2=check, 4=analysis, 8=error
    Ok(())
}
```

### 4. collect_files

Like cclinter. Filter `.py` extension.

```rust
fn collect_files(paths: &[PathBuf], ignore: &IgnoreMatcher) -> Result<Vec<PathBuf>, ...> {
    // ... walkdir, match .py extension
}
```

### 5. print_diff

Copy `print_diff` from cclinter.

## Tests

Create `tests/cli_mode_tests.rs`:

```rust
#[test]
fn no_args_shows_error() {
    // pylinter with no args → error
}

#[test]
fn check_mode_detects_issues() {
    // prepare .py file with format issues
    // cargo run -- --check <file> → exit 1
}

#[test]
fn diff_mode_shows_changes() {
    // prepare .py file needing format
    // cargo run -- --diff <file> → output diff
}
```

## Verify

```bash
cargo build
echo "x=1" > /tmp/test.py
cargo run -- /tmp/test.py
cargo run -- --check /tmp/test.py
cargo run -- --diff /tmp/test.py
```
