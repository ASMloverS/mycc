# mycc

User-domain AI coding configs + C language toolchain.

## Structure

```
mycc/
├── tools/
│   ├── sync_config.py         # interactive config sync
│   ├── test_sync_config.py    # sync_config tests
│   ├── sync_config.yaml       # sync source manifest
│   ├── statusline.mjs         # opencode status bar (ctx%, branch, cost)
│   └── linter/cclinter/       # Rust C linter (see below)
├── custom-harness/claude/     # Claude harness: agents, commands, skills, bin
├── skills/claude/             # Claude skill dispatch
├── docs/                      # design docs
│   ├── harness-design.md
│   └── cclinter/
├── AGENTS.md                  # agentic edit rules
├── CLAUDE.md                  # Claude-specific rules + gitmoji convention
├── sync.bat / sync.sh         # sync_config launchers (Win / Unix)
└── README.md
```

## sync_config.py

Interactive copy from `~/.claude`, `~/.agents`, `~/.config/opencode` → repo.

### Usage

```bash
python tools/sync_config.py            # interactive select → copy
python tools/sync_config.py --dry-run  # preview only
```

Requires `pip install inquirer`.

### Skip

~30 items skipped by name: `.credentials.json`, `node_modules`, `__pycache__`, `cache`, `sessions`, etc. See `SKIP` set.

### Behavior

- Copy: file → overwrite, dir → `rmtree` + `copytree`.
- Source discovery: scans `custom-harness/<source>/` subdirectories (agents, commands, skills, bin).
- Returns 4-tuples: `(Path, src_key, sub_cat, from_harness)`.

## cclinter

Rust-based C language linter at `tools/linter/cclinter/`.

**Architecture:** Regex/text-matching parser. Three engines: formatter → checker → analyzer.

**Tech stack:** Rust stable, clap, serde_yaml, regex, rayon, walkdir, globset, colored, similar.

### Implemented

| Engine | Feature | Status |
|--------|---------|--------|
| Formatter | Encoding (BOM/CRLF/trailing ws) | Done |
| Formatter | Indent (tab→2-space, brace-level) | Done |
| Formatter | Spacing (ops, comma, paren, semicolon) | Done |
| Formatter | YAML config loading | Done |
| Checker | Diagnostic framework (clang-tidy output) | Done |
| Analyzer | Analysis level framework (basic/strict/deep) | Done |

See `docs/cclinter/index.md` for full task tracking.

### Build & Test

```bash
cd tools/linter/cclinter
cargo build
cargo test
```

## Rules

See `AGENTS.md`:
- Grep → Read ±20 lines → Edit. ≤100 lines/Edit.
- UTF-8, LF, no trailing whitespace.
- Gitmoji commit format: `gitmoji type(scope): desc`.
