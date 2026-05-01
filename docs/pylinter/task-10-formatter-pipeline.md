# Task 10: Formatter Pipeline Integration + E2E Tests

> Status: ⬜ Not started
> Depends: Task 06, Task 07, Task 08, Task 09
> Output: `format_source()` pipeline works end-to-end, tests pass

## Goal

Wire formatter modules into complete pipeline. Write E2E tests.

## Steps

### 1. formatter/mod.rs

```rust
pub mod blank_lines;
pub mod binary_op;
pub mod comment_style;
pub mod encoding;
pub mod import_sort;
pub mod indent;
pub mod line_length;
pub mod trailing_ws;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn format_source(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
    encoding::fix_encoding(source, config)?;
    trailing_ws::fix_trailing_ws(source, config)?;
    indent::fix_indent(source, config)?;
    blank_lines::fix_blank_lines(source, config)?;
    // 以下模块在 Phase 2 实现, 此处为占位
    // import_sort::fix_import_sort(source, config)?;
    // comment_style::fix_comment_style(source, config)?;
    // line_length::fix_line_length(source, config)?;
    // binary_op::fix_binary_op(source, config)?;
    Ok(vec![])
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
x=1
def foo(  ):
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
