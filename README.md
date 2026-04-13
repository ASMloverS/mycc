# mycc

Sync toolset for user-domain AI coding configs.

## Structure

```
mycc/
├── sync_config.py        # interactive config sync
├── agents/               # agent defs
├── skills/               # skill defs
├── commands/             # custom commands
└── AGENTS.md             # edit rules
```

## sync_config.py

Interactive copy from `~/.claude`, `~/.agents`, `~/.config/opencode` → cwd.

### Usage

```bash
python sync_config.py            # interactive select → copy
python sync_config.py --dry-run  # preview only
```

Requires `pip install inquirer`.

### Skip

~30 items skipped by name: `.credentials.json`, `node_modules`, `__pycache__`, `cache`, `sessions`, etc. See `SKIP` set.

### Behavior

- Copy: file → overwrite, dir → `rmtree` + `copytree`.
- Target by category: agents→`./agents/`, commands→`./commands/`, skills→`./skills/`, config→`./`.

## Rules

See `AGENTS.md`:
- Grep → Read ±20 lines → Edit. ≤100 lines/Edit.
- UTF-8, LF, no trailing whitespace.
