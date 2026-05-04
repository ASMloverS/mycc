# Task 10: Formatter Pipeline Integration + E2E Tests

> Status: ✅ Done
> Depends: Task 06, Task 07, Task 08, Task 09
> Output: `format_source()` pipeline works end-to-end, tests pass

## Goal

Wire formatter modules into complete pipeline. Write E2E tests.

## Steps

### 1. formatter/mod.rs

```rust
pub mod blank_lines;
pub mod encoding;
pub mod indent;
pub mod trailing_ws;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::FormatConfig;
use crate::cst::CSTSource;

pub fn format_source(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<Vec<Diagnostic>, String> {
    encoding::fix_encoding(source, config).map_err(|e| e.to_string())?;
    if let Ok(mut cst) = CSTSource::parse(&source.content) {
        indent::fix_indent(&mut cst, config).map_err(|e| e.to_string())?;
        blank_lines::fix_blank_lines(&mut cst, config).map_err(|e| e.to_string())?;
        trailing_ws::fix_trailing_ws(&mut cst, config).map_err(|e| e.to_string())?;
        source.content = cst.regenerate();
    }
    Ok(Vec::new())
}
```

### 2. Wire cli.rs

Call `format_source` inside `cli.rs` `run()`:

```rust
// inside file processing loop:
let diags = crate::formatter::format_source(&mut source, config_ref)?;
```

### 3. Test fixtures

**tests/fixtures/input/dirty.py**:
```python
import os
x = 1
def foo():
 	pass
class Bar:
  def baz(self):
    pass
```

**tests/fixtures/expected/dirty.py**:
```python
import os
x = 1


def foo():
    pass


class Bar:

    def baz(self):
        pass
```

### 4. E2E test

```rust
// tests/formatter_tests.rs

#[test]
fn format_dirty_file() {
    let input = fs::read_to_string("tests/fixtures/input/dirty.py").unwrap();
    let expected = fs::read_to_string("tests/fixtures/expected/dirty.py").unwrap();
    let mut source = SourceFile::from_string(&input, PathBuf::from("dirty.py"));
    let config = FormatConfig::default();
    format_source(&mut source, &config).unwrap();
    assert_eq!(source.content, expected);
}
```

### 5. Extra test cases

```rust
#[test]
fn idempotent_formatting() {
    // format twice → same result
    let input = "...";
    let config = FormatConfig::default();
    let mut src1 = SourceFile::from_string(input, PathBuf::from("test.py"));
    format_source(&mut src1, &config).unwrap();
    let mut src2 = SourceFile::from_string(&src1.content, PathBuf::from("test.py"));
    format_source(&mut src2, &config).unwrap();
    assert_eq!(src1.content, src2.content);
}

#[test]
fn preserves_already_formatted() {
    // already-formatted code stays unchanged
}
```

## Verify

```bash
cargo test -- formatter
```
