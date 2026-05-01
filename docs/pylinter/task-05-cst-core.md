# Task 05: CST Core — tokenize + parse + CSTSource + regenerate

> Status: ⬜ Not started
> Depends: Task 03
> Output: Python src → tokenize → CSTSource → regenerate = identical text (roundtrip)

## Goal

Build CST core modules. Parse Python src into structured IR. Lossless roundtrip. Foundation for all formatters/checkers.

## Refs

- Design doc §2: CSTSource / CSTLine / IndentInfo / LocatedToken
- `rustpython-parser` crate: `tokenize()` + `parse()` API

## Steps

### 1. Verify rustpython-parser API

Write small smoke test:

```rust
use rustpython_parser::tokenize;

#[test]
fn verify_tokenizer() {
    let src = "def foo():\n    pass\n";
    let tokens: Vec<_> = tokenize(src).collect();
    // Print tokens. Confirm COMMENT/INDENT/DEDENT/NEWLINE/NL exist.
}
```

> If `rustpython-parser` 0.3 API differs, adjust crate name or use git source. May need:
> ```toml
> rustpython-parser = { git = "https://github.com/RustPython/Parser", rev = "..." }
> ```

### 2. cst/tokens.rs — Token wrapper

```rust
use rustpython_parser::Tok;

#[derive(Clone, Debug)]
pub struct LocatedToken {
    pub tok: Tok,
    pub start: (usize, usize),  // (line 1-based, column 0-based)
    pub end: (usize, usize),
}

pub fn tokenize_source(source: &str) -> Result<Vec<LocatedToken>, String> {
    // Call rustpython_parser::tokenize
    // Map (Location, Tok, Location) → LocatedToken
    // Filter ERRORTOKEN → return Err if found
}
```

### 3. cst/lines.rs — Group by line

```rust
#[derive(Clone, Debug)]
pub struct IndentInfo {
    pub level: usize,
    pub raw: String,
    pub width: usize,
    pub uses_tabs: bool,
}

#[derive(Clone, Debug)]
pub struct CSTLine {
    pub num: usize,           // 1-based line number
    pub indent: IndentInfo,
    pub tokens: Vec<LocatedToken>,
    pub trailing_ws: String,
    pub comment: Option<String>,
    pub is_blank: bool,
}

pub fn build_lines(tokens: &[LocatedToken], source: &str) -> Vec<CSTLine> {
    // 1. Group tokens by line
    // 2. Extract indent per line (leading whitespace → first non-ws char)
    // 3. Extract trailing whitespace
    // 4. Extract COMMENT token → comment field
    // 5. Mark is_blank
}
```

### 4. cst/mod.rs — CSTSource

```rust
use rustpython_parser::ast::Mod;

pub struct CSTSource {
    pub lines: Vec<CSTLine>,
    pub ast: Mod,
    pub source: String,  // original source
}

impl CSTSource {
    pub fn parse(source: &str) -> Result<Self, String> {
        let tokens = tokenize_source(source)?;
        let ast = rustpython_parser::parse(source, Mode::Module, "<input>")
            .map_err(|e| e.to_string())?;
        let lines = build_lines(&tokens, source);
        Ok(Self { lines, ast, source: source.to_string() })
    }
}
```

### 5. cst/generate.rs — Source rebuild

```rust
impl CSTSource {
    pub fn regenerate(&self) -> String {
        // Iterate self.lines. Per line:
        //   indent.raw + token text + trailing_ws + comment + "\n"
        // Concat all.
        //
        // Key: lossless roundtrip required.
    }
}
```

### 6. AST node mapping (scaffold — future tasks fill in)

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum AstNodeKind {
    FunctionDef,
    ClassDef,
    Import,
    ImportFrom,
    // ... expand later
}

pub fn map_ast_to_lines(ast: &Mod, lines: &mut [CSTLine]) {
    // Walk AST. Mark each line's ast_nodes by node position.
    // This task: scaffold only. Future tasks fill mapping.
}
```

## Tests

```rust
#[test]
fn roundtrip_simple() {
    let src = "x = 1\ny = 2\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_function() {
    let src = "def foo():\n    return 42\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_with_comments() {
    let src = "# comment\nx = 1  # inline\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_multiline() {
    let src = concat!(
        "class Foo:\n",
        "    def bar(self):\n",
        "        pass\n",
        "\n",
        "    def baz(self):\n",
        "        return 1\n",
    );
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_preserves_indent() {
    let src = "if True:\n    if True:\n        pass\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn parse_error_returns_err() {
    let src = "def foo(:\n";
    assert!(CSTSource::parse(src).is_err());
}
```

## Verify

```bash
cargo test -- cst
```

All roundtrip tests pass → CST core correct + lossless.
