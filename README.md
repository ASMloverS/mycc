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
│       ├── cclinter/               # Rust C linter
│       └── pylinter/               # Rust Python linter
├── custom-harness/
│   ├── claude/                     # Claude harness: agents, commands, skills, bin
│   └── opencode/                   # OpenCode harness: AGENTS.md
├── skills/claude/                  # Claude skill dispatch
├── docs/
│   ├── harness/                    # harness design + optimization docs
│   ├── configs/                    # config evolution + permissions docs
│   ├── cclinter/                   # cclinter design + task docs
│   └── pylinter/                   # pylinter design + task docs
├── sync.bat / sync.sh              # sync_config launchers (Win / Unix)
└── README.md
```

## sync_config.py

Bidirectional sync: user-domain (`~/.claude`, `~/.config/opencode`) ↔ repo.

```bash
python tools/sync_config.py pull             # interactive select → copy to repo
python tools/sync_config.py pull --dry-run   # preview only
python tools/sync_config.py install          # copy from repo → user domain
python tools/sync_config.py install-agents   # install agents + set model
```

Requires `pip install pyyaml`. ~30 items skipped by name (`.credentials.json`, `node_modules`, etc.).

## cclinter

Rust C linter at `tools/linter/cclinter/`. Regex/text-matching: formatter → checker → analyzer.
Three phases complete (17+9+5 tasks). See `docs/cclinter/index.md`.

```bash
cd tools/linter/cclinter && cargo build && cargo test
```

## pylinter

Rust Python 3.14+ linter at `tools/linter/pylinter/`. CST-based (rustpython-parser), same design as cclinter.
Three phases complete (10+11+5 tasks). See `docs/pylinter/pylinter-tasks.md`.

```bash
cd tools/linter/pylinter && cargo build && cargo test
```

## Rules

See `custom-harness/opencode/AGENTS.md`:
- Grep → Read ±20 lines → Edit. ≤100 lines/Edit.
- UTF-8, LF, no trailing whitespace.
- Gitmoji commit format: `gitmoji type(scope): desc`.
