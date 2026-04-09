# mycc

Sync toolset for user-domain AI coding configs.

## Structure

```
mycc/
├── sync_config.py        # interactive config sync
├── agents/               # agent defs
│   ├── code-implementer.md
│   ├── code-reviewer.md
│   └── doc-corrector.md
├── skills/               # skill defs
│   ├── doc-refine/
│   ├── doc-sync/
│   └── doc-write/
├── commands/             # custom commands (empty)
└── AGENTS.md             # edit rules
```

## sync_config.py

Interactive copy from `~/.claude`, `~/.agents`, `~/.config/opencode` → cwd.

### Sources

| Source | Path | What |
|--------|------|------|
| claude | `~/.claude` | agents/, commands/, CLAUDE.md, settings.json |
| agents | `~/.agents` | skills/ |
| opencode | `~/.config/opencode` | skills/, opencode*.json, package.json |

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

## Agents

| Agent | Model | Purpose |
|-------|-------|---------|
| code-implementer | claude-sonnet-4-6 | TDD impl + bugfix |
| code-reviewer | claude-opus-4-6 | spec-driven code review |
| doc-corrector | claude-haiku-4-5 | align docs to code |

## Skills

| Skill | Purpose |
|-------|---------|
| doc-refine | compress docs: EN→caveman, CN→文言文 |
| doc-sync | align docs with code |
| doc-write | write docs in ultra-compressed style |

## Rules

See `AGENTS.md`:
- Grep → Read ±20 lines → Edit. ≤100 lines/Edit.
- UTF-8, LF, no trailing whitespace.
