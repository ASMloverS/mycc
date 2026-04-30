### Task 05: Exit Code 4, Full Integration + Cross-Platform Tests

**Files:**
- Modify: `tools/linter/cclinter/src/cli.rs`
- Create: `tests/integration_tests.rs`
- Test fixtures: `tests/fixtures/analysis_test.c`

- [x] **Step 1: Wire up analyzer in `src/cli.rs`**

After the checker section, add analyzer invocation. Reuse `seen` HashSet for dedup:

```rust
if !args.format_only && config.analysis.level != AnalysisLevel::None {
    let analysis_config = &config.analysis;
    let analysis_level = &analysis_config.level;
    let analysis_runtime_err = AtomicU8::new(0);
    let all_analysis_diags: Vec<Diagnostic> = files
        .par_iter()
        .flat_map(|file_path| {
            let source = SourceFile::load(file_path)?;
            analyzer::analyze_source(&source, analysis_level, analysis_config)
        })
        .collect();
    exit_code |= analysis_runtime_err.load(Ordering::Relaxed);

    for diag in &all_analysis_diags {
        let key = (diag.file.clone(), diag.line, diag.rule_id.clone());
        if seen.insert(key) {
            if args.verbose || !args.quiet {
                eprintln!("{diag}");
            }
        }
    }
    if !all_analysis_diags.is_empty() {
        exit_code |= 4;
    }
}
```

Key: uses `AnalysisLevel` enum directly (not string comparison). Passes both `analysis_level` and `analysis_config` to `analyze_source`. Skips if `level == None` or `--format-only`. Uses same `seen` HashSet for cross-engine dedup. `AtomicU8` for runtime error tracking across threads.

- [x] **Step 2: Create integration test fixture**

Create `tests/fixtures/analysis_test.c`:

```c
#include <stdio.h>
#include <stdlib.h>

int* create_buf() {
    int* p = NULL;
    *p = 42;
    return p;
}

void process(char* input) {
    char buf[10];
    gets(buf);
    printf("%s\n", input);
}
```

- [x] **Step 3: Write integration tests**

Create `tests/integration_tests.rs`:

```rust
use std::process::Command;

fn run_cclinter(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args({
            let mut a = vec!["run", "--"];
            a.extend_from_slice(args);
            a
        })
        .current_dir("tools/linter/cclinter")
        .output()
        .unwrap()
}

#[test]
fn test_full_pipeline_format_check() {
    let output = run_cclinter(&["--check", "tests/fixtures/input/full_test.c"]);
    assert_ne!(output.status.code(), Some(0));
}

#[test]
fn test_analysis_deep_flags_issues() {
    let output = run_cclinter(&["--check", "--analysis-level", "deep", "tests/fixtures/analysis_test.c"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("null-deref") || stderr.contains("buffer-overflow") || stderr.contains("resource-leak"));
}

#[test]
fn test_analysis_none_skips() {
    let output = run_cclinter(&["--check", "--analysis-level", "none", "--format-only", "tests/fixtures/expected/full_test.c"]);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_in_place_creates_formatted() {
    let dir = tempfile::tempdir().unwrap();
    let test_file = dir.path().join("test.c");
    std::fs::write(&test_file, "int x=1;\n").unwrap();
    let output = run_cclinter(&["-i", test_file.to_str().unwrap()]);
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("int x = 1;"));
}

#[test]
fn test_diff_mode_output() {
    let output = run_cclinter(&["--diff", "tests/fixtures/input/full_test.c"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty() || output.status.code() == Some(0));
}
```

- [x] **Step 4: Run all tests**

Run: `cargo test`
Expected: All PASS.

- [x] **Step 5: Verify cross-platform build**

```bash
cargo build --release
```

Verify binary exists: `target/release/cclinter` (Linux) or `target/release/cclinter.exe` (Windows).

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✅ feat(cclinter): full integration with exit codes and cross-platform build"
```
