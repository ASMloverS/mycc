### Task 16: rayon Parallel File Processing

**Files:**
- Modify: `tools/linter/cclinter/src/cli.rs`

- [ ] **Step 1: Write failing test**

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

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --test cli_mode_tests test_parallel`
Expected: FAIL — parallel not implemented.

- [ ] **Step 3: Implement parallel processing in `src/cli.rs`**

Replace the serial file loop with rayon:

```rust
use rayon::prelude::*;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let tool_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let config = crate::config::find_config(args.config.as_ref(), &tool_dir)?;
    let files = collect_files(&args.paths)?;

    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new().num_threads(jobs).build_global().ok();
    }

    let results: Vec<(PathBuf, String, String)> = files
        .par_iter()
        .filter_map(|file_path| {
            let content = std::fs::read_to_string(file_path).ok()?;
            let source = crate::common::source::SourceFile::from_string(&content, file_path.clone());
            let formatted = crate::formatter::format_source(&source, &config);
            Some((file_path.clone(), content, formatted.content))
        })
        .collect();

    let mut exit_code = 0u8;
    for (file_path, original, formatted) in &results {
        if args.check {
            if original != formatted {
                eprintln!("{}: formatting issues found", file_path.display());
                exit_code |= 1;
            }
        } else if args.diff {
            let diff = TextDiff::from_lines(original, formatted);
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                print!("{}{}", sign, change);
            }
        } else if args.in_place {
            if original != formatted {
                std::fs::write(file_path, formatted)?;
                if !args.quiet {
                    eprintln!("formatted {}", file_path.display());
                }
            }
        } else {
            print!("{}", formatted);
        }
    }

    if exit_code != 0 {
        std::process::exit(exit_code as i32);
    }
    Ok(())
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "⚡ perf(cclinter): rayon parallel file processing with -j config"
```
