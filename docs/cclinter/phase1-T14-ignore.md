### Task 14: `.cclinterignore` Support

**Files:**
- Modify: `tools/linter/cclinter/src/ignore.rs`
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tests/ignore_tests.rs`

- [x] **Step 1: Write failing tests**

Create `tests/ignore_tests.rs`:

```rust
use cclinter::ignore::IgnoreMatcher;
use std::path::Path;

#[test]
fn test_ignore_pattern() {
    let patterns = vec!["*.generated.c".to_string(), "build/".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("foo.generated.c")));
    assert!(matcher.is_ignored(Path::new("build/main.c")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_ignore_comment_lines() {
    let content = "# comment\n*.bak\n\nvendor/\n";
    let matcher = IgnoreMatcher::from_string(content);
    assert!(matcher.is_ignored(Path::new("test.c.bak")));
    assert!(matcher.is_ignored(Path::new("vendor/foo.c")));
}

#[test]
fn test_empty_matcher() {
    let matcher = IgnoreMatcher::from_string("");
    assert!(!matcher.is_ignored(Path::new("anything.c")));
}
```

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test ignore_tests`
Expected: FAIL.

- [x] **Step 3: Implement `src/ignore.rs`**

```rust
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};

pub struct IgnoreMatcher {
    set: GlobSet,
}

impl IgnoreMatcher {
    pub fn from_patterns(patterns: &[String]) -> Self {
        let mut builder = GlobSetBuilder::new();
        for pat in patterns {
            if let Ok(glob) = Glob::new(pat) {
                let _ = builder.add(glob);
            }
        }
        let set = builder.build().unwrap_or_else(|_| GlobSet::empty());
        Self { set }
    }

    pub fn from_string(content: &str) -> Self {
        let patterns: Vec<String> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.to_string())
            .collect();
        Self::from_patterns(&patterns)
    }

    pub fn from_file(path: &Path) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        Self::from_string(&content)
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.set.is_match(path_str.as_ref())
    }
}
```

- [x] **Step 4: Update `src/cli.rs` to use ignore**

In the `run()` function, after collecting files, add ignore filtering:

```rust
let ignore_path = tool_dir.join(".cclinterignore");
let matcher = crate::ignore::IgnoreMatcher::from_file(&ignore_path);
let files: Vec<PathBuf> = collected_files
    .into_iter()
    .filter(|f| !matcher.is_ignored(f))
    .collect();
```

(Exact integration depends on file collection logic from T01.)

- [x] **Step 5: Run tests**

Run: `cargo test --test ignore_tests`
Expected: All PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): .cclinterignore file support"
```
