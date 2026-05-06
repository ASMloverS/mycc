# pylinter design

Python 3.14+ linter: format + style check + static analysis

## 1. Arch

Peer of cclinter. Self-contained. No shared crate.

```
tools/linter/pylinter/
├── Cargo.toml
├── .pylinter.yaml
├── .gitattributes
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── ignore.rs
│   ├── cst/
│   │   ├── mod.rs           # tokenize + AST → CSTSource
│   │   ├── tokens.rs        # token wrap + pos map
│   │   ├── lines.rs         # CSTLine: per-line token/indent/AST
│   │   └── generate.rs      # CSTSource → String
│   ├── common/
│   │   ├── mod.rs
│   │   ├── source.rs        # SourceFile
│   │   ├── diag.rs          # Diagnostic + Severity
│   │   └── string_utils.rs
│   ├── formatter/
│   │   ├── mod.rs           # pipeline dispatch
│   │   ├── encoding.rs      # UTF-8, LF, strip BOM
│   │   ├── trailing_ws.rs   # strip trailing ws
│   │   ├── indent.rs        # indent normalize (AST-aware)
│   │   ├── blank_lines.rs   # blank line normalize (AST-aware)
│   │   ├── import_sort.rs   # PEP 8 / isort sort (AST-based)
│   │   ├── line_length.rs   # line len check + wrap (token-aware)
│   │   ├── binary_op.rs     # binary op line break
│   │   └── comment_style.rs # comment style unify (#)
│   ├── checker/
│   │   ├── mod.rs
│   │   ├── naming.rs        # naming check (configurable)
│   │   ├── complexity.rs    # complexity: lines + nesting
│   │   ├── magic_number.rs  # magic numbers
│   │   ├── unused_import.rs # unused imports
│   │   ├── prohibited.rs    # fn/module blacklist
│   │   └── docstring.rs     # docstring presence
│   └── analyzer/
│       ├── mod.rs            # 3-tier dispatch: basic/strict/deep
│       ├── basic.rs          # mutable default, missing self, bare except, == None
│       ├── strict.rs         # redundant pass, empty f-string, redundant if-return
│       └── deep.rs           # unreachable code, unused var, shadow builtin
└── tests/
    ├── fixtures/
    │   ├── input/
    │   └── expected/
    ├── integration_tests.rs
    ├── formatter_tests.rs
    ├── checker_tests.rs
    └── analyzer_tests.rs
```

## 2. CST core: Token-CST hybrid

### Data flow

```
Source string
  → tokenize() → Vec<Token>  (COMMENT/INDENT/DEDENT/NEWLINE/NL)
  → parse()    → AST (Mod)
  → CSTSource  (token stream + AST, organized by line)
  → Formatter transforms
  → regenerate() → formatted string
```

### CSTSource struct

```rust
struct CSTSource {
    lines: Vec<CSTLine>,
    ast: Mod,
    line_ending: LineEnding,
}

struct CSTLine {
    num: usize,
    indent: IndentInfo,
    tokens: Vec<LocatedToken>,
    trailing_ws: String,
    comment: Option<String>,
    is_blank: bool,
    ast_nodes: Vec<AstNodeKind>,
}

struct IndentInfo {
    level: usize,
    raw: String,
    width: usize,
    uses_tabs: bool,
}

struct LocatedToken {
    tok: Tok,
    start: (usize, usize),
    end: (usize, usize),
}
```

### Format pipeline

Same as cclinter `format_source`:

```
encoding → indent → blank_lines → import_sort
→ comment_style → line_length → trailing_ws → binary_op → generate
```

## 3. Deps

```toml
[package]
name = "pylinter"
version = "0.1.0"
edition = "2021"
description = "Python 3.14+ linter: format + style check + static analysis"

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

## 4. Config

```yaml
format:
  indent_width: 4
  use_tabs: false
  column_limit: 120
  line_ending: lf
  encoding: utf-8
  trailing_whitespace: strip
  blank_lines_before_class: 2
  blank_lines_before_function: 2
  blank_lines_inside_class: 1
  max_consecutive_blank_lines: 2
  import_sorting: pep8
  binary_op_line_break: before
  comment_style: hash

check:
  naming:
    function: snake_case
    class: pascal_case
    constant: upper_snake_case
    variable: snake_case
    module: snake_case
  complexity:
    max_function_lines: 50
    max_class_lines: 300
    max_file_lines: 1000
    max_nesting_depth: 4
  magic_number:
    enabled: true
    allowed: [0, 1, -1, 2]
  unused_import:
    enabled: true
  prohibited:
    use_default: true
    extra: []
    remove: []
  docstring:
    module: true
    class: true
    function: true

analysis:
  level: basic
```

## 5. CLI

Same as cclinter:

```
pylinter <paths...> [OPTIONS]

Options:
  --config <PATH>         Config file path
  -i, --in-place          Format in place
  --check                 Check mode (exit 1 if issues)
  --diff                  Show diff
  --format-only           Only format, skip check/analysis
  --analysis-level <LEVEL> Override analysis level
  -j, --jobs <N>          Parallel jobs
  --exclude <PATTERN>     Exclude patterns
  -q, --quiet             Quiet mode
  -v, --verbose           Verbose mode
```

- Files: `.py`
- Ignore: `.pylinterignore`
- Exit codes: bit flags (1=format, 2=check, 4=analysis, 8=error)

## 6. Formatter

| Module | Input | Method |
|---|---|---|
| `encoding` | raw string | strip BOM, `\r\n`→`\n`, `\r`→`\n` |
| `trailing_ws` | CSTLine | `trailing_ws = ""` |
| `indent` | CSTLine + INDENT/DEDENT | rebuild indent by AST level |
| `blank_lines` | CSTLine + AST boundary | add/remove blank lines around class/fn def |
| `import_sort` | Import/ImportFrom nodes | group+sort: stdlib → third-party → local |
| `comment_style` | COMMENT tokens | unify `# ` |
| `line_length` | CSTLine tokens | wrap at token boundary |
| `binary_op` | BinOp + tokens | move op to line start/end |

## 7. Checker

| Module | AST node | Check |
|---|---|---|
| `naming` | FunctionDef, ClassDef, Name, Import | configurable naming style |
| `complexity` | FunctionDef, ClassDef | lines + nesting depth |
| `magic_number` | Constant (numeric) | unnamed numeric literal |
| `unused_import` | Import, ImportFrom + Name refs | import unused |
| `prohibited` | Call, Attribute | configurable blacklist |
| `docstring` | Module, ClassDef, FunctionDef first Expr | presence |

## 8. Analyzer 3-tier

| Level | Rule | rule_id |
|---|---|---|
| **basic** | mutable default arg | `bugprone-mutable-default` |
| | missing self | `bugprone-missing-self` |
| | bare except | `bugprone-bare-except` |
| `== None` not `is None` | `bugprone-none-comparison` |
| **strict** | redundant pass | `readability-unnecessary-pass` |
| | empty f-string | `readability-empty-fstring` |
| | redundant if-return | `readability-simplify-if-return` |
| **deep** | unreachable code | `deadcode-unreachable` |
| | unused variable | `deadcode-unused-variable` |
| | shadow builtin | `bugprone-shadow-builtin` |

## 9. Priority

### Phase 1: Skeleton + core formatting

- Cargo structure + CLI + config + common
- CST core (tokenize + parse + CSTSource + regenerate)
- encoding + trailing_ws + indent + blank_lines
- Integration test framework

### Phase 2: Advanced formatting + Checker

- import_sort + line_length + binary_op + comment_style
- 6 checker modules

### Phase 3: Analyzer

- basic / strict / deep tiers

## 10. Open questions

- `rustpython-parser` crate version/API → verify at impl time (may need git dep)
- `.ipynb` support?
- `--init` command needed?
