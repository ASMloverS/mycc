### Task 15: `--diff` / `--check` / `-i` Modes

**Files:**
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tests/cli_mode_tests.rs`

- [ ] **Step 1: Write failing tests**

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

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test cli_mode_tests`
Expected: FAIL — modes not implemented yet.

- [ ] **Step 3: Implement mode logic in `src/cli.rs`**

```rust
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let tool_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let config = crate::config::find_config(args.config.as_ref(), &tool_dir)?;
    let files = collect_files(&args.paths)?;
    let mut exit_code = 0u8;

    for file_path in &files {
        let content = std::fs::read_to_string(file_path)?;
        let source = crate::common::source::SourceFile::from_string(&content, file_path.clone());
        let formatted = crate::formatter::format_source(&source, &config);

        if args.check {
            if content != formatted.content {
                eprintln!("{}: formatting issues found", file_path.display());
                exit_code |= 1;
            }
        } else if args.diff {
            let diff = TextDiff::from_lines(&content, &formatted.content);
            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                print!("{}{}", sign, change);
            }
        } else if args.in_place {
            if content != formatted.content {
                std::fs::write(file_path, &formatted.content)?;
                if !args.quiet {
                    eprintln!("formatted {}", file_path.display());
                }
            }
        } else {
            print!("{}", formatted.content);
        }
    }

    if exit_code != 0 {
        std::process::exit(exit_code as i32);
    }
    Ok(())
}

fn collect_files(paths: &[PathBuf]) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path.clone());
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry?;
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if ext == "c" || ext == "h" {
                        files.push(p.to_path_buf());
                    }
                }
            }
        }
    }
    Ok(files)
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test cli_mode_tests`
Expected: All PASS.

- [ ] **Step 5: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): --check, --diff, -i mode support"
```
