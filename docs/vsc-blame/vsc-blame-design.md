# vsc-blame design

CLI tool: blame file/line ownership via git or svn. Parse traceback, diff, or direct file:line input. Smart aggregation to find the responsible person.

## 1. Arch

Self-contained Rust binary. Peer of pylinter/cclinter.

```
tools/vsc-blame/
├── Cargo.toml
├── .gitattributes
├── src/
│   ├── main.rs              # entry, dispatch subcommand
│   ├── lib.rs
│   ├── cli.rs               # clap def + run()
│   ├── config.rs            # YAML config loading
│   ├── vcs/
│   │   ├── mod.rs           # VcsBackend trait
│   │   ├── git.rs           # git blame impl
│   │   ├── svn.rs           # svn blame impl
│   │   └── detect.rs        # auto-detect VCS type
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── traceback_py.rs  # Python traceback parse
│   │   ├── traceback_cpp.rs # C/C++ stack trace parse
│   │   └── diff.rs          # unified diff parse
│   ├── blame.rs             # BlameEntry + aggregation logic
│   ├── reporter/
│   │   ├── mod.rs           # Reporter trait
│   │   ├── text.rs          # terminal colored text
│   │   ├── json.rs          # JSON output
│   │   ├── markdown.rs      # Markdown table
│   │   └── html.rs          # HTML report
│   └── util.rs
└── tests/
    ├── fixtures/
    │   ├── traceback/
    │   └── diff/
    └── integration_tests.rs
```

## 2. CLI

```
vsc-blame [GLOBAL OPTIONS] <COMMAND>

Commands:
  blame      Blame specified file/lines (default when no command)
  traceback  Parse traceback/stack trace and blame each frame
  diff       Parse diff and blame changed lines

Global Options:
  --vcs <git|svn>               Override VCS detection (default: auto)
  --format <text|json|md|html>  Output format (default: text)
  --config <PATH>               Config file (default: .vsc-blame.yaml)
  --no-color                    Disable colored output
  -q, --quiet
  -v, --verbose
  -h, --help
  -V, --version
```

### blame

```
vsc-blame blame <FILE>[:LINE[-ENDLINE]] [OPTIONS]

Options:
  --all       Blame entire file (default when no line spec)
  --summary   Show aggregated summary instead of per-line detail
```

No subcommand defaults to blame: `vsc-blame foo.py:10` == `vsc-blame blame foo.py:10`.

`FILE:START-END` is a **closed interval [START, END], 1-based**; internally stored as `LineSpec::Range(START, END)`.

### traceback

```
vsc-blame traceback [TEXT | -f FILE | --stdin]

Options:
  -f, --file <PATH>   Read traceback from file
  --stdin             Read from stdin
```

Trailing argument treated as traceback text directly.

### diff

```
vsc-blame diff [OPTIONS]

Options:
  -f, --file <PATH>   Read diff from file
  --stdin             Read from stdin
  --base <REF>        Git base ref (e.g. HEAD~3, main)
  --head <REF>        Git head ref (default: HEAD)
  --base-rev <REV>    SVN base revision
  --head-rev <REV>    SVN head revision
```

## 3. Core data structures

```rust
enum VcsKind { Git, Svn }

enum LineSpec {
    All,
    Single(usize),
    Range(usize, usize),        // closed [start, end], 1-based
    Multi(Vec<(usize, usize)>), // multiple segments for traceback / diff
}

struct BlameEntry {
    file: String,
    line: usize,
    author: String,
    author_mail: String,
    author_time: NaiveDateTime,
    vcs: VcsKind,
    commit_id: String,    // git: 40-hex hash; svn: revision number string
    summary: String,      // commit message summary
    content: String,      // source line content
}

struct BlameResult {
    entries: Vec<BlameEntry>,
    summary: Vec<AuthorSummary>,
    suggested_responsible: Option<String>,
    uncommitted_lines: Vec<(String, usize)>, // (file, line) filtered before aggregation
}

struct AuthorSummary {
    author: String,
    mail: String,
    commit_count: usize,
    score: f64,
    latest_time: NaiveDateTime,
    latest_commit: String,
    files: Vec<String>,
    lines: Vec<usize>,
}

trait Reporter {
    fn render(&self, r: &BlameResult, out: &mut dyn Write) -> Result<()>;
}
```

JSON `commit` field carries a VCS prefix: `"git:abc1234"` or `"svn:r42"`.

## 4. VCS backend

```rust
trait VcsBackend {
    fn name(&self) -> &str;
    fn blame_file(&self, file: &str, lines: &LineSpec) -> Result<Vec<BlameEntry>>;
    fn diff_revisions(&self, base: &str, head: &str) -> Result<Vec<FileDiff>>;
}

struct FileDiff {
    file: String,
    hunks: Vec<Hunk>,
}

struct Hunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    added_lines: Vec<usize>,   // line numbers in new file
}
```

### Auto-detection (vcs/detect.rs)

1. Walk up from cwd, find `.git/` dir -> Git
2. Walk up from cwd, find `.svn/` dir -> SVN
3. Neither found -> error
4. `--vcs` flag overrides detection

### Git backend

Execute `git blame --porcelain <file>` and parse porcelain format output.

### SVN backend

1. `svn blame <file>` — produces line → revision mapping.
2. `svn log -r MIN:MAX --xml <file>` — one call to fetch all commit metadata in the revision range; merge into entries locally.

Do **not** issue per-line `svn log` calls (N-trip cost).

### Command execution rules

All external processes (`git`, `svn`) **must** use `std::process::Command::new(...).arg(...)` — no shell interpolation.

- File paths starting with `-` are prefixed with `--` (e.g., `-- -oddname.rs`).
- ref / revision values must match `[A-Za-z0-9._/~^@:-]+`; reject with exit code 2 otherwise.
- Never construct command strings by concatenation.

## 5. Parsers

### Python traceback (parser/traceback_py.rs)

```
Traceback (most recent call last):
  File "example.py", line 42, in <module>
    foo()
  File "example.py", line 10, in foo
    bar()
```

Use `Regex::find_iter` on the full input to collect **all** matches of `File "(.+?)", line (\d+)` — this covers chained exceptions separated by `During handling of the above exception…` / `The above exception was the direct cause…` blocks.

### C/C++ stack trace (parser/traceback_cpp.rs)

Formats supported:
- GDB: `#0  foo () at example.c:42`
- MSVC: `example.cpp(42): foo()`
- addr2line: `foo at /path/example.c:42`
- backtrace_symbols: `./prog(foo+0x1a) [0x...]` — **no file:line info; skip frame and emit a warning. addr2line integration is out of scope for v0.1.**

### Diff parser (parser/diff.rs)

Parse unified diff format. Extract `+` lines (added/modified) with file and line number, then blame those lines.

**Diff mode constraints:**

| Flag group | VCS | Mutually exclusive with |
|---|---|---|
| `--base` / `--head` | git only | `--base-rev`, `--head-rev`, `stdin/-f` |
| `--base-rev` / `--head-rev` | svn only | `--base`, `--head`, `stdin/-f` |
| `stdin` / `-f` | both | `--base`, `--head`, `--base-rev`, `--head-rev` |

Enforcement: clap `conflicts_with`. Incompatible flag + `--vcs` combination is rejected with exit code 2.

Working-tree blame target when using a diff file:
- git: blame `HEAD` version
- svn: blame `BASE` version

## 6. Aggregation

Smart sorting to find "most likely responsible person":

**Pre-aggregation filter:** Remove entries where `commit_id == "0".repeat(40)` or `author == "Not Committed Yet"` (git working-tree lines). Collect them into `BlameResult.uncommitted_lines`; they do not affect scoring.

**Alias resolution:** author names/emails mapped via config `author_aliases` (case-insensitive, matches both name and email) before grouping.

**Scoring:**

```
recency_score(t) = exp(-Δt / τ),  τ = 90 days,  Δt = now - latest_commit_time
commit_norm      = commit_count / total_entries_in_result  ∈ [0, 1]
score            = W_COMMIT * commit_norm + W_RECENCY * recency_score
W_COMMIT = 0.4,  W_RECENCY = 0.6
```

Top scorer = `suggested_responsible`.

## 7. Output formats

### Terminal colored text (default)

```
File: src/main.py
───────────────────────────────────────────────────
Line 10 | zhangsan | 2024-03-15 | abc1234 | fix: add feature
        | import os
───────────────────────────────────────────────────

Summary (2 lines):
  #1 zhangsan (zhangsan@company.com) - 1 commit, latest 2024-03-15
  #2 lisi     (lisi@company.com)     - 1 commit, latest 2024-04-02

Suggested responsible: lisi (most recent change)
```

### JSON

```json
{
  "entries": [{"file": "...", "line": 10, "author": "...", ...}],
  "summary": [{"author": "...", "commit_count": 1, "score": 0.35}],
  "suggested_responsible": "lisi"
}
```

### Markdown

```markdown
| Line | Author | Date | Commit | Summary |
|------|--------|------|--------|---------|
| 10   | zhangsan | 2024-03-15 | abc1234 | fix: add feature |
```

### HTML

Self-contained HTML file with sortable table.

## 8. Config

`.vsc-blame.yaml`:

```yaml
vcs: auto              # auto | git | svn
format: text           # text | json | md | html
output: null           # output file path, null=stdout
no_color: false

author_aliases:
  zhangsan:
    - zs
    - zhangsan@old.com
  lisi:
    - ls

weights:
  commit_count: 0.4
  recency: 0.6

defaults:
  blame_summary: false
  blame_all: false
```

Priority: CLI args > env vars > config file > built-in defaults.

Env var mapping: `VSC_BLAME_VCS`, `VSC_BLAME_FORMAT`, `VSC_BLAME_NO_COLOR`, `VSC_BLAME_OUTPUT`.

## 9. Deps

```toml
[package]
name = "vsc-blame"
version = "0.1.0"
edition = "2021"
description = "Blame tool: git/svn blame with traceback/diff parsing and smart aggregation"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
regex = "1"
chrono = "0.4"
colored = "2"

[dev-dependencies]
tempfile = "3"
```

## 10. Exit codes & stderr

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | generic error |
| 2 | CLI usage error (bad flags, invalid arg format) |
| 3 | VCS not detected or not installed |
| 4 | file not found or not under VCS |
| 5 | empty result (no lines to blame) |

stderr lines are prefixed `[ERROR]` or `[WARN]`. `--quiet` suppresses `[WARN]` but **not** `[ERROR]`.

## 11. Phases

### Phase 1: Skeleton + blame core

- Cargo structure + CLI + config + util
- VCS auto-detection + Git backend
- Blame data structures
- Terminal text reporter

### Phase 2: Parsers + SVN

- Python traceback parser
- C/C++ stack trace parser
- Diff parser
- SVN backend
- JSON reporter

### Phase 3: Aggregation + advanced output

- Author aggregation + smart scoring
- Author alias resolution
- Markdown reporter
- HTML reporter
- Integration tests
