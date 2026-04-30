### Task 16: rayon Parallel File Processing

**Files:**
- Modify: `tools/linter/cclinter/src/cli.rs`

- [x] **Step 1: Write failing test**

Add to `tests/cli_mode_tests.rs`:

```rust
#[test]
fn test_parallel_processes_all_files() {
    let output = Command::new("cargo")
        .args(["run", "--", "--check", "-j2", "tests/fixtures/input/"])
        .current_dir("tools/linter/cclinter")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("formatting issues") || output.status.code() == Some(1));
}
```

- [x] **Step 2: Run test to verify failure**

Run: `cargo test --test cli_mode_tests test_parallel`
Expected: FAIL — parallel not implemented.

- [x] **Step 3: Implement parallel processing in `src/cli.rs`**

Three parallel sections in `run()`:

1. **Formatter** (always runs):
```rust
let results: Vec<FormatResult> = files
    .par_iter()
    .map(|file_path| {
        let mut source = SourceFile::load(file_path)?;
        let diags = formatter::format_source(&mut source, config_ref)?;
        Ok((file_path.clone(), source.original, source.content, diags))
    })
    .collect();
```

2. **Checker** (unless `--format-only`):
```rust
let all_diags: Vec<Diagnostic> = files
    .par_iter()
    .flat_map(|file_path| {
        let source = SourceFile::load(file_path)?;
        checker::check_source(&source, check_config)
    })
    .collect();
```

3. **Analyzer** (unless `--format-only` or `analysis.level == None`):
```rust
let all_analysis_diags: Vec<Diagnostic> = files
    .par_iter()
    .flat_map(|file_path| {
        let source = SourceFile::load(file_path)?;
        analyzer::analyze_source(&source, analysis_level, analysis_config)
    })
    .collect();
```

Thread pool setup: `rayon::ThreadPoolBuilder::new().num_threads(jobs).build_global()`. Runtime errors tracked via `AtomicU8` with `Ordering::Relaxed`.

- [x] **Step 4: Run tests**

Run: `cargo test`
Expected: All PASS.

- [x] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "⚡ perf(cclinter): rayon parallel file processing with -j config"
```
