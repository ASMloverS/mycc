### Task 09: Checker Integration, Exit Code 2, Tests

**Files:**
- Modify: `tools/linter/cclinter/src/checker/mod.rs`
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tests/checker_integration_tests.rs`

- [ ] **Step 1: Wire up `check_source` in `src/checker/mod.rs`**

```rust
pub mod naming;
pub mod include_guard;
pub mod complexity;
pub mod magic_number;
pub mod unused;
pub mod prohibited;
pub mod forward_decl;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::Config;

pub fn check_source(source: &SourceFile, config: &Config) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    if let Some(ref naming) = config.check.naming {
        diags.extend(naming::check_naming(source, naming.variable.as_deref().unwrap_or("snake_case"), "variable"));
        diags.extend(naming::check_naming(source, naming.function.as_deref().unwrap_or("snake_case"), "function"));
        if let Some(ref style) = naming.macro {
            diags.extend(naming::check_naming(source, style, "macro"));
        }
    }
    diags.extend(include_guard::check_includes(source));
    if let Some(ref complexity) = config.check.complexity {
        diags.extend(complexity::check_complexity(
            source,
            complexity.max_function_lines.unwrap_or(100),
            complexity.max_file_lines.unwrap_or(2000),
            complexity.max_nesting_depth.unwrap_or(5),
        ));
    }
    if let Some(ref mn) = config.check.magic_number {
        diags.extend(magic_number::check_magic_numbers(
            source,
            mn.enabled.unwrap_or(true),
            &mn.allowed.clone().unwrap_or_default(),
        ));
    }
    diags.extend(unused::check_unused(source));
    if let Some(ref pf) = config.check.prohibited_functions {
        let use_default = pf.use_default.unwrap_or(true);
        let extra = pf.extra.as_deref().unwrap_or(&[]);
        let remove = pf.remove.as_deref().unwrap_or(&[]);
        diags.extend(prohibited::check_prohibited(source, use_default, extra, remove));
    }
    diags.extend(forward_decl::check_forward_decls(source));
    diags
}
```

- [ ] **Step 2: Update `src/cli.rs` — checker invocation + exit code 2 + dedup**

After the formatter loop, add checker invocation. Use `HashSet` for dedup (see design doc "Diagnostic Deduplication"):

```rust
use std::collections::HashSet;

let mut seen: HashSet<(String, usize, String)> = HashSet::new();

if !args.format_only {
    let check_results: Vec<Vec<Diagnostic>> = files
        .par_iter()
        .filter_map(|file_path| {
            let content = std::fs::read_to_string(file_path).ok()?;
            let source = crate::common::source::SourceFile::from_string(&content, file_path.clone());
            Some(crate::checker::check_source(&source, &config))
        })
        .collect();

    for diags in &check_results {
        for d in diags {
            let key = (d.file.clone(), d.line, d.rule_id.clone());
            if seen.insert(key) {
                if args.verbose || !args.quiet {
                    eprintln!("{}", d);
                }
            }
        }
        if !diags.is_empty() {
            exit_code |= 2;
        }
    }
}
```

- [ ] **Step 3: Write integration test**

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

- [ ] **Step 4: Create test fixture**

Create `tests/fixtures/checker_test.c`:

```c
int BadName = 42;
void bad_fn() { strcpy(buf, src); }
```

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: All PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): checker integration with exit code 2"
```
