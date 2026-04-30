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
use std::path::Path;

pub struct IgnoreMatcher {
    set: GlobSet,
}

impl IgnoreMatcher {
    pub fn from_patterns(patterns: &[String]) -> Self {
        // Expands patterns: root-only (/prefix), directory (/suffix),
        // simple names → **/name/** + **/name
    }
    pub fn from_string(content: &str) -> Self { ... }
    pub fn from_file(path: &Path) -> Self { ... }
    pub fn is_ignored(&self, path: &Path) -> bool { ... }
    pub fn is_empty(&self) -> bool { ... }
}
```

Pattern expansion rules:
- `/` prefix → root-only match
- `/` suffix or directory name → `{pat}/**`
- Simple name (no glob, no separator) → `**/{pat}/**` + `**/{pat}`
- Negation (`!`) patterns → warning, skipped

- [x] **Step 4: Update `src/cli.rs` to use ignore**

In `cli.rs`, `build_ignore_matcher` combines `--exclude` patterns with `.cclinterignore` patterns:

```rust
fn build_ignore_matcher(args: &Args) -> crate::ignore::IgnoreMatcher {
    let mut patterns: Vec<String> = args.exclude.clone();
    let ignore_path = std::path::Path::new(".cclinterignore");
    if ignore_path.exists() {
        if let Ok(content) = std::fs::read_to_string(ignore_path) {
            // Filter: skip empty, comments, negation patterns
            patterns.extend(file_patterns);
        }
    }
    crate::ignore::IgnoreMatcher::from_patterns(&patterns)
}
```

File collection uses `walkdir` to recursively find `.c` and `.h` files, filtered by `IgnoreMatcher`.

- [x] **Step 5: Run tests**

Run: `cargo test --test ignore_tests`
Expected: All PASS.

- [x] **Step 6: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): .cclinterignore file support"
```
