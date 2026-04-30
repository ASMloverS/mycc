### Task 09: Checker Integration, Exit Code 2, Tests

**Files:**
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tests/checker_integration_tests.rs`

- [x] **Step 1: Wire up `check_source` in `src/checker/mod.rs`**

```rust
pub fn check_source(source: &SourceFile, config: &CheckConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(naming::check_naming(source, config.naming.function.as_str(), "function"));
    diags.extend(naming::check_naming(source, config.naming.r#macro.as_str(), "macro"));
    diags.extend(naming::check_naming(source, config.naming.variable.as_str(), "variable"));
    diags.extend(naming::check_naming(source, config.naming.r#type.as_str(), "type"));
    diags.extend(naming::check_naming(source, config.naming.constant.as_str(), "constant"));
    diags.extend(include_guard::check_include_guard(source, &config.include_guard));
    diags.extend(complexity::check_complexity(source, &config.complexity));
    diags.extend(magic_number::check_magic_number(source, &config.magic_number));
    diags.extend(unused::check_unused(source, &config.unused));
    diags.extend(prohibited::check_prohibited(
        source,
        config.prohibited_functions.use_default,
        &config.prohibited_functions.extra,
        &config.prohibited_functions.remove,
    ));
    diags.extend(forward_decl::check_forward_decl(source));
    diags
}
```

Key: takes `&CheckConfig` (not `&Config`). All sub-checkers use their respective config structs from `CheckConfig`. Naming checks all 5 kinds. `NamingStyle.as_str()` converts enum to string slice.

- [x] **Step 2: Update `src/cli.rs` — checker invocation + exit code 2 + dedup**

After the formatter loop, add checker invocation. Uses `HashSet` for dedup across checker and analyzer:

```rust
let mut seen: HashSet<(String, usize, String)> = HashSet::new();

if !args.format_only {
    let check_config = &config.check;
    let runtime_err = AtomicU8::new(0);
    let all_diags: Vec<Diagnostic> = files
        .par_iter()
        .flat_map(|file_path| {
            let source = SourceFile::load(file_path)?;
            checker::check_source(&source, check_config)
        })
        .collect();
    exit_code |= runtime_err.load(Ordering::Relaxed);

    for diag in &all_diags {
        let key = (diag.file.clone(), diag.line, diag.rule_id.clone());
        if seen.insert(key) {
            if args.verbose || !args.quiet {
                eprintln!("{diag}");
            }
        }
    }
    if !all_diags.is_empty() {
        exit_code |= 2;
    }
}
```

Key: `check_source` takes `&CheckConfig`. `SourceFile::load()` used (reads from disk). `AtomicU8` for runtime error tracking. Dedup key: `(file, line, rule_id)`. Output suppressed only when both `!verbose && quiet`.

- [x] **Step 3: Write integration test**

Create `tests/checker_integration_tests.rs`:

```rust
use std::process::Command;

#[test]
fn test_checker_flags_issues() {
    let output = Command::new("cargo")
        .args(["run", "--", "--check", "tests/fixtures/checker_test.c"])
        .current_dir("tools/linter/cclinter")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("warning") || stderr.contains("error"));
}
```

- [x] **Step 4: Create test fixture**

Create `tests/fixtures/checker_test.c`:

```c
int BadName = 42;
void bad_fn() { strcpy(buf, src); }
```

- [x] **Step 5: Run all tests**

Run: `cargo test`
Expected: All PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): checker integration with exit code 2"
```
