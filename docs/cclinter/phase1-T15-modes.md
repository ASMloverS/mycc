### Task 15: `--diff` / `--check` / `-i` Modes

**Files:**
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tests/cli_mode_tests.rs`

- [x] **Step 1: Write failing tests**

Create `tests/cli_mode_tests.rs`:

```rust
use std::process::Command;

#[test]
fn test_check_mode_exits_1_on_issues() {
    let output = Command::new("cargo")
        .args(["run", "--", "--check", "tests/fixtures/input/encoding_test.c"])
        .current_dir("tools/linter/cclinter")
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(0));
}

#[test]
fn test_check_mode_exits_0_on_clean() {
    let output = Command::new("cargo")
        .args(["run", "--", "--check", "tests/fixtures/expected/encoding_test.c"])
        .current_dir("tools/linter/cclinter")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_diff_mode_shows_changes() {
    let output = Command::new("cargo")
        .args(["run", "--", "--diff", "tests/fixtures/input/encoding_test.c"])
        .current_dir("tools/linter/cclinter")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-") || stdout.contains("+"));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test cli_mode_tests`
Expected: FAIL — modes not implemented yet.

- [x] **Step 3: Implement mode logic in `src/cli.rs`**

```rust
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config(args.config.as_ref())?;
    if let Some(ref level) = args.analysis_level {
        config.analysis.level = level.clone();
    }
    let ignore = build_ignore_matcher(&args);
    let files = collect_files(&args.paths, &ignore)?;

    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new().num_threads(jobs).build_global().ok();
    }

    let config_ref = &config.format;
    let results: Vec<FormatResult> = files
        .par_iter()
        .map(|file_path| {
            let mut source = SourceFile::load(file_path)?;
            let diags = formatter::format_source(&mut source, config_ref)?;
            Ok((file_path.clone(), source.original, source.content, diags))
        })
        .collect();

    let mut exit_code = 0u8;
    for result in &results {
        // --check: compare original vs formatted, exit_code |= 1
        // --diff: print unified diff via `similar` crate
        // -i: write back if changed
        // default: print formatted to stdout
    }
    // ... checker and analyzer sections
}
```

Key: `format_source` takes `&mut SourceFile`. `SourceFile` has `original` and `content` fields for change tracking. Three mutually exclusive modes: `--check`, `--diff`, `-i`.

- [x] **Step 4: Run tests**

Run: `cargo test --test cli_mode_tests`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): --check, --diff, -i mode support"
```
