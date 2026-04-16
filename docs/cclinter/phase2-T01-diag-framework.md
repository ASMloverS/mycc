### Task 01: clang-tidy Diagnostic Output + Rule Trait

**Files:**
- Modify: `tools/linter/cclinter/src/common/diag.rs`
- Modify: `tools/linter/cclinter/src/common/rule.rs`
- Test: `tests/diag_tests.rs`

- [ ] **Step 1: Write failing tests**

Create `tests/diag_tests.rs`:

```rust
use cclinter::common::diag::{Diagnostic, Severity};

#[test]
fn test_diag_format_warning() {
    let d = Diagnostic::new("foo.c".into(), 10, 5, Severity::Warning, "unused-var", "Variable 'x' is unused");
    let s = d.to_string();
    assert!(s.contains("foo.c:10:5:"));
    assert!(s.contains("warning"));
    assert!(s.contains("[unused-var]"));
}

#[test]
fn test_diag_format_error() {
    let d = Diagnostic::new("bar.c".into(), 1, 1, Severity::Error, "prohibited-fn", "Use of gets() is prohibited");
    let s = d.to_string();
    assert!(s.contains("error"));
    assert!(s.contains("[prohibited-fn]"));
}

#[test]
fn test_diag_with_source_line() {
    let d = Diagnostic::new_with_source("test.c".into(), 3, 10, Severity::Warning, "magic-number", "Magic number 42", "  int x = 42;");
    let s = d.to_string();
    assert!(s.contains("int x = 42;"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test diag_tests`
Expected: FAIL.

- [ ] **Step 3: Expand `src/common/diag.rs`**

```rust
use colored::*;

#[derive(Clone, Debug)]
pub enum Severity {
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
    pub fn new(file: String, line: usize, col: usize, severity: Severity, rule_id: &str, message: &str) -> Self {
        Self {
            file,
            line,
            col,
            severity,
            rule_id: rule_id.to_string(),
            message: message.to_string(),
            source_line: None,
        }
    }

    pub fn new_with_source(file: String, line: usize, col: usize, severity: Severity, rule_id: &str, message: &str, source: &str) -> Self {
        Self {
            file,
            line,
            col,
            severity,
            rule_id: rule_id.to_string(),
            message: message.to_string(),
            source_line: Some(source.to_string()),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (sev_str, sev_colored) = match self.severity {
            Severity::Warning => ("warning", "warning".yellow()),
            Severity::Error => ("error", "error".red()),
        };
        write!(
            f,
            "{}:{}:{}: {}: {} [{}]",
            self.file, self.line, self.col, sev_colored, self.message, self.rule_id
        )?;
        if let Some(ref src) = self.source_line {
            write!(f, "\n  {}", src)?;
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Expand `src/common/rule.rs`**

```rust
use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;

pub trait Rule {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, source: &SourceFile) -> Vec<Diagnostic>;
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test diag_tests`
Expected: All PASS.

- [ ] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): clang-tidy diagnostic output format with rule trait"
```
