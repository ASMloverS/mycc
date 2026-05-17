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
struct BlameEntry {
    file: String,
    line: usize,
    author: String,
    author_mail: String,
    author_time: NaiveDateTime,
    commit: String,       // git: hash, svn: revision
    summary: String,      // commit message summary
    content: String,      // source line content
}

struct BlameResult {
    entries: Vec<BlameEntry>,
}

struct AuthorSummary {
    author: String,
    mail: String,
    commit_count: usize,
    latest_time: NaiveDateTime,
    latest_commit: String,
    files: Vec<String>,
    lines: Vec<usize>,
}
```

## 4. VCS backend

```rust
trait VcsBackend {
    fn name(&self) -> &str;
    fn blame_file(&self, file: &str, lines: Option<Range<usize>>) -> Result<Vec<BlameEntry>>;
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

Execute `svn blame <file>` with optional `-r` range, then `svn log -r <rev>` for detail.

## 5. Parsers

### Python traceback (parser/traceback_py.rs)

```
Traceback (most recent call last):
  File "example.py", line 42, in <module>
    foo()
  File "example.py", line 10, in foo
    bar()
```

Regex: `File "(.+?)", line (\d+)` -> `Vec<(file, line)>`

### C/C++ stack trace (parser/traceback_cpp.rs)

Formats supported:
- GDB: `#0  foo () at example.c:42`
- MSVC: `example.cpp(42): foo()`
- addr2line: `foo at /path/example.c:42`
- backtrace_symbols: `./prog(foo+0x1a) [0x...]`

### Diff parser (parser/diff.rs)

Parse unified diff format. Extract `+` lines (added/modified) with file and line number, then blame those lines.

Strategy for uncommitted changes:
- `--base`/`--head` mode: blame the `--head` version
- diff file mode: blame current working tree version

## 6. Aggregation

Smart sorting to find "most likely responsible person":

1. Group BlameEntry by author (after alias resolution)
2. Score = `commit_count * W_COMMIT + recency_score * W_RECENCY`
3. `recency_score` = exponential decay based on `latest_time` (more recent = higher)
4. Default weights: `W_COMMIT = 0.4`, `W_RECENCY = 0.6`
5. Top scorer = suggested responsible

Alias resolution: author names/emails mapped via config `author_aliases` before grouping.

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

## 10. Phases

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
