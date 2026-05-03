# mycc

User-domain AI coding configs + C/Python linter toolchain.

## Structure

```
mycc/
├── tools/
│   ├── sync_config.py              # bidirectional config sync (pull/install)
│   ├── test_sync_config.py         # sync_config tests
│   ├── sync_config.yaml            # sync source manifest
│   ├── statusline.mjs              # opencode status bar (ctx%, branch, cost)
│   └── linter/
│       ├── cclinter/               # Rust C linter (see below)
│       └── pylinter/               # Rust Python linter (see below)
├── custom-harness/
│   ├── claude/                     # Claude harness: agents, commands, skills, bin
│   └── opencode/                   # OpenCode harness: AGENTS.md
├── skills/claude/                  # Claude skill dispatch
├── docs/
│   ├── harness-design.md
│   ├── dispatch-optimization.md
│   ├── harness-stabilization.md
│   ├── cclinter/                   # cclinter design + task docs
│   └── pylinter/                   # pylinter design + task docs
├── sync.bat / sync.sh              # sync_config launchers (Win / Unix)
└── README.md
```

## sync_config.py

Bidirectional sync between user-domain (`~/.claude`, `~/.config/opencode`) ↔ repo.

### Usage

```bash
python tools/sync_config.py pull                # interactive select → copy to repo
python tools/sync_config.py pull --dry-run      # preview only
python tools/sync_config.py install             # copy from repo → user domain
python tools/sync_config.py install-agents      # install agents + set model
```

Requires `pip install pyyaml`.

### Skip

~30 items skipped by name: `.credentials.json`, `node_modules`, `__pycache__`, `cache`, `sessions`, etc. See `DEFAULT_SKIP` set.

### Behavior

- **pull**: interactive select → copy from user domain to repo. Harness sources scanned from `custom-harness/<source>/`.
- **install**: copy from repo → user domain.
- **install-agents**: batch install agent files + optional model override.
- Copy: file → overwrite, dir → `rmtree` + `copytree`.

## cclinter

Rust-based C language linter at `tools/linter/cclinter/`.

**Architecture:** Regex/text-matching parser. Three engines: formatter → checker → analyzer.

**Tech stack:** Rust stable, clap, serde_yaml, regex, rayon, walkdir, globset, colored, similar.

### Status

All three phases complete (17 formatting tasks, 9 checker tasks, 5 analyzer tasks). See `docs/cclinter/index.md`.

### Build & Test

```bash
cd tools/linter/cclinter
cargo build
cargo test
```

## pylinter

Rust-based Python 3.14+ linter at `tools/linter/pylinter/`.

**Architecture:** CST-based (rustpython-parser). Peer of cclinter — same three-engine design: formatter → checker → analyzer.

**Tech stack:** Rust stable, clap, serde_yaml, regex, rayon, walkdir, globset, colored, similar, rustpython-parser.

### Status

Phase 1 scaffold + core formatting in progress (7/10 tasks done). See `docs/pylinter/pylinter-tasks.md`.

### Build & Test

```bash
cd tools/linter/pylinter
cargo build
cargo test
```

## Rules

See `custom-harness/opencode/AGENTS.md`:
- Grep → Read ±20 lines → Edit. ≤100 lines/Edit.
- UTF-8, LF, no trailing whitespace.
- Gitmoji commit format: `gitmoji type(scope): desc`.
