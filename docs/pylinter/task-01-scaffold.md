# Task 01: Project scaffold — Cargo.toml + main.rs + lib.rs

> Status: ⬜ Not started
> Deps: none
> Output: `cargo build` passes, `cargo run -- --help` works

## Goal

Pylinter Cargo skeleton. Compiles. `--help` runs.

## Steps

### 1. Create dirs

```
tools/linter/pylinter/
├── src/
│   ├── main.rs
│   └── lib.rs
├── Cargo.toml
└── .gitattributes
```

### 2. Cargo.toml

```toml
[package]
name = "pylinter"
version = "0.1.0"
edition = "2021"
description = "Python 3.12+ linter: format + style check + static analysis"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
regex = "1"
rayon = "1"
walkdir = "2"
globset = "0.4"
colored = "2"
similar = "2"
rustpython-parser = "0.3"

[dev-dependencies]
tempfile = "3"
```

> `rustpython-parser` crate name/version → verify at impl time. May need `rustpython_parser` (underscore) or git source.

### 3. .gitattributes

```
* text=auto eol=lf
```

### 4. src/main.rs

Follow cclinter `src/main.rs`:

```rust
use pylinter::cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("{e}");
        std::process::exit(8);
    }
}
```

### 5. src/lib.rs

Stub modules only (later tasks fill in):

```rust
pub mod cli;
pub mod common;
pub mod config;
pub mod cst;
pub mod formatter;
pub mod checker;
pub mod analyzer;
pub mod ignore;
```

### 6. Stub modules

Create `mod.rs` or file per module. Empty exports. Must compile:

- `src/cli.rs` — `pub fn run()` → `Ok(())`
- `src/config.rs` — `pub struct Config` + `impl Default`
- `src/common/mod.rs` — empty
- `src/cst/mod.rs` — empty
- `src/formatter/mod.rs` — empty
- `src/checker/mod.rs` — empty
- `src/analyzer/mod.rs` — empty
- `src/ignore.rs` — empty struct

## Verify

```bash
cd tools/linter/pylinter
cargo build
cargo run -- --help
```

Expect: compiles. `--help` prints pylinter usage.
