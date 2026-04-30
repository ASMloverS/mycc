# cclinter — C Linter Design

Rust C linter: format + style check + static analysis. `tools/linter/cclinter/`.

## Tech Stack

| Item | Choice |
|------|--------|
| Lang | Rust stable |
| Parse | Regex/text |
| YAML | serde_yaml |
| CLI | clap |
| Parallel | rayon (`-j`) |
| Config | `.cclinter.yaml` |
| Ignore | `.cclinterignore` |
| Output | clang-tidy style (`colored` crate) |
| Severity | Note / Warning / Error |
| Exit codes | Bitwise OR combinable |
| Rule IDs | Descriptive naming |
| Dist | `cargo build` → standalone binary |
| Test | Unit + snapshot |
| Modules | formatter / checker / analyzer / config / cli |

## CLI

```
cclinter [OPTIONS] <PATHS...>

--config <FILE>           Config (search: --config → CWD → ancestor dirs → tool dir → built-in)
-i, --in-place            Modify in-place (mutually exclusive with --check, --diff)
--check                   Check only (CI). Exit 1 if issues. (mutually exclusive with -i, --diff)
--diff                    Show diff, no modify (mutually exclusive with -i, --check)
--format-only             Formatter only, skip checker + analyzer.
                          Compatible with --check, --diff, -i.
--analysis-level <LVL>    Override analysis.level: none | basic | strict | deep
-j, --jobs <N>            Parallelism (default: CPU count), must be >= 1
--exclude <PATTERN>       Extra exclude patterns (append to .cclinterignore)
-q, --quiet               Suppress diagnostic output
-v, --verbose             Show config, diagnostics, and processing details
```

### Config Precedence

CLI > `.cclinter.yaml` > built-in defaults.

`--analysis-level` overrides `analysis.level` in YAML.

### Config Search Order

1. `--config <FILE>` — explicit, fail if missing
2. `.cclinter.yaml` in CWD
3. Walk parent dirs → first `.cclinter.yaml` found
4. `.cclinter.yaml` in tool binary dir (`current_exe().parent()`)
5. Built-in defaults (`Config::default()`)

### Exit Codes

Bitwise OR:

| Code | Meaning |
|------|---------|
| 0 | No issues |
| 1 | Formatting (`--check`) |
| 2 | Style violations |
| 4 | Analysis issues |
| 8 | Runtime errors |

`exit(3)` = format(1) + style(2). `exit(7)` = format + style + analysis.

### Diagnostic Dedup

Checker + analyzer may flag same line. Dedup via:

```rust
HashSet<(file: String, line: usize, rule_id: String)>
```

Skip if `(file, line, rule_id)` seen. Different `rule_id` on same line → emit both.

## Directory Structure

```
tools/linter/cclinter/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── config.rs
│   ├── ignore.rs
│   ├── lib.rs
│   ├── formatter/
│   │   ├── mod.rs
│   │   ├── encoding.rs          # UTF-8 / LF / trailing ws
│   │   ├── indent.rs            # Brace-level indent (2-space default)
│   │   ├── spacing.rs
│   │   ├── braces.rs            # Attach / Breakout / AttachBreakout
│   │   ├── blank_lines.rs
│   │   ├── comments.rs          # /* */ → // (all)
│   │   ├── pointer_style.rs     # int* p (left) or int *p (right)
│   │   ├── switch_indent.rs     # switch-case indentation
│   │   ├── line_length.rs       # Column-limit wrap
│   │   ├── alignment.rs         # Struct/enum field alignment
│   │   └── include_sort.rs      # Google 3-group
│   ├── checker/
│   │   ├── mod.rs
│   │   ├── naming.rs            # snake_case / UPPER_SNAKE_CASE / PascalCase
│   │   ├── include_guard.rs
│   │   ├── complexity.rs
│   │   ├── magic_number.rs
│   │   ├── unused.rs
│   │   ├── prohibited.rs
│   │   └── forward_decl.rs
│   ├── analyzer/
│   │   ├── mod.rs
│   │   ├── basic.rs
│   │   ├── strict.rs
│   │   └── deep.rs
│   └── common/
│       ├── mod.rs
│       ├── diag.rs              # clang-tidy output (Note/Warning/Error)
│       ├── source.rs            # SourceFile + mask_string_literals, strip_line_comment
│       ├── rule.rs
│       └── string_utils.rs      # split_outside_strings
├── tests/
│   ├── analyzer_tests.rs
│   ├── checker_integration_tests.rs
│   ├── checker_tests.rs
│   ├── cli_mode_tests.rs
│   ├── common/
│   ├── complexity_tests.rs
│   ├── config_tests.rs
│   ├── diag_tests.rs
│   ├── fixtures/{input,expected}/
│   ├── formatter_tests.rs
│   ├── forward_decl_tests.rs
│   ├── ignore_tests.rs
│   ├── include_guard_tests.rs
│   ├── integration_tests.rs
│   ├── magic_number_tests.rs
│   ├── naming_tests.rs
│   ├── prohibited_tests.rs
│   ├── snapshot_tests.rs
│   └── unused_tests.rs
└── .cclinter.yaml
```

## Config (`.cclinter.yaml`)

```yaml
format:
  column_limit: 120
  indent_width: 2
  use_tabs: false
  pointer_alignment: left          # left | right
  brace_style: attach              # attach | breakout | attach-breakout
  switch_case_indent: true
  blank_lines_before_function: 1
  blank_lines_after_include: 1
  max_consecutive_blank_lines: 2
  space_before_paren: false
  spaces_around_operators: true
  include_sorting: google          # google | none
  comment_style: double_slash      # double_slash | preserve
  line_ending: lf                  # lf | crlf | native
  encoding: utf-8

check:
  naming:
    function: snake_case
    variable: snake_case
    constant: upper_snake_case
    type: pascal_case
    macro: upper_snake_case
  complexity:
    max_function_lines: 100
    max_file_lines: 2000
    max_nesting_depth: 5
  magic_number:
    enabled: true
    allowed: [0, 1, -1, 2]
  unused:
    enabled: true
  include_guard:
    style: pragma_once             # pragma_once | ifndef
  prohibited_functions:
    use_default: true               # true = include built-in list
    extra: []                        # Append to effective list
    remove: []                       # Remove from effective list
    # Built-in (not user-overridable): strcpy, strcat, sprintf, vsprintf, gets, scanf
    # Effective = (built-in if use_default) + extra - remove

analysis:
  level: basic                     # none | basic | strict | deep (default: basic)
```

## Key API Signatures

### SourceFile (`common/source.rs`)

```rust
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub original: String,
}

impl SourceFile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn Error>>;
    pub fn from_string(content: &str, path: PathBuf) -> Self;
    pub fn lines(&self) -> Vec<&str>;          // method, not a field
    pub fn line_count(&self) -> usize;
    pub fn is_modified(&self) -> bool;
}
```

### Formatter Pipeline

All formatters take `(&mut SourceFile, &FormatConfig) -> Result<(), Box<dyn Error>>`.

```rust
pub fn format_source(source: &mut SourceFile, config: &FormatConfig) -> Result<Vec<Diagnostic>, Box<dyn Error>>;
```

Pipeline order: encoding → indent → spacing → braces → blank_lines → comments → pointer_style → switch_indent → line_length → alignment → include_sort.

### Checker

```rust
pub fn check_source(source: &SourceFile, config: &CheckConfig) -> Vec<Diagnostic>;
```

### Analyzer

```rust
pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel, config: &AnalysisConfig) -> Vec<Diagnostic>;
```

Levels are cumulative: Strict = Basic + Strict; Deep = Basic + Strict + Deep.

### Config (`config.rs`)

All config structs use `#[serde(default)]` with non-Optional fields. Enums implement `Default`, `clap::ValueEnum`, `Serialize`, `Deserialize`. Config loading: `load_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn Error>>`.

## Comment Conversion

All `/* */` → `//`:
- `/* text */` → `// text`
- Multi-line → `//` per line
- Copyright blocks → `//` per line
- Existing `//` → unchanged

## Google C Style Rules

Fn signature line-break · pointer alignment (`int* p`) · blank lines · switch-case indent · struct/enum alignment · operator/comma/paren spacing · continuation alignment

## Style Check Rules

**Naming**: fn/var `snake_case`, const/macro `UPPER_SNAKE_CASE`, type `PascalCase`
**Include guard**: missing guard, duplicate includes
**Complexity**: fn lines, file lines, nesting depth
**Magic number**: literal detection + allowlist
**Unused**: vars, macros (configurable via `unused.enabled`)
**Prohibited fns**: default list + YAML extend/remove
**Forward decl**: missing in headers

## Static Analysis Levels

| Level | Scope |
|-------|-------|
| none | Off |
| basic | Implicit conv, missing return, uninit hints |
| strict | + Suspicious casts, dead branches, resource leaks |
| deep | + Buffer overflow patterns, null deref patterns |

Default: `basic`. CLI `--analysis-level` overrides `analysis.level` in YAML.

## Dependencies

```toml
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

[dev-dependencies]
tempfile = "3"
```

## Development Phases

### Phase 1 — Formatting

1. Skeleton + clap + modules + source abstraction
2. UTF-8 BOM removal, CRLF→LF, trailing ws strip
3. Tab→2-space, brace-level indent
4. Operator/comma/paren/semicolon spacing
5. Brace attach style
6. Blank line normalization
7. `/* */` → `//` (all)
8. Pointer alignment: `int *p` → `int* p`
9. switch-case indent
10. Continuation + struct/enum alignment
11. 120-col line wrap
12. #include Google 3-group sort
13. YAML config load + directory lookup
14. `.cclinterignore`
15. `--diff` / `--check` / `-i`
16. rayon parallel
17. Unit + snapshot tests

### Phase 2 — Style Checking

1. clang-tidy diag framework + rule trait
2. Naming: snake_case / UPPER_SNAKE_CASE / PascalCase
3. Include guard + duplicate detection
4. Complexity: fn/file lines, nesting
5. Magic number detection + allowlist
6. Unused: vars, macros, params
7. Prohibited fn + YAML extend
8. Forward decl check
9. Exit code 2

### Phase 3 — Static Analysis

1. Level framework: basic / strict / deep
2. basic: implicit conv, missing return, uninit
3. strict: suspicious casts, dead code, resource leaks
4. deep: buffer overflow, null deref patterns
5. Exit code 4
6. Integration + cross-platform tests (Win11, Debian 12)

## Cross-Platform

- Win11 → `cclinter.exe`, Debian 12 → `cclinter`
- Platform path handling for config lookup
- `line_ending` config: `lf` (default), `crlf`, `native` (currently encoding.rs normalizes to LF regardless)
- `.cclinterignore` uses gitignore-style patterns (negation `!` not supported)
- `encoding` config field exists but currently always normalizes to UTF-8/LF
