---
name: vsc-commit
description: Use when committing changes via git or svn. Supports both VCS, smart file filtering, gitmoji (git) or plain (svn) msgs, CWD-scoped only. Trigger with /vsc-commit.
---

# vsc-commit

Commit CWD changes. Git (default) or svn (`--svn`). Auto-filter junk. Smart msg gen.

## Params

| Param | Req | Default | Desc |
|-------|-----|---------|------|
| `<msg>` | no | AI-gen | User intent hint; combined w/ diff → final msg |
| `--svn` | no | false | → svn mode |

## Detect VCS

1. `--svn` flag → svn
2. `.git/` exists in CWD → git
3. `.svn/` exists in CWD → svn
4. Neither → error, stop

## Get Changes

**Git:** `git status --porcelain .`

**SVN:** `svn status .` → parse col1:

| Code | Meaning | Pre-commit action |
|------|---------|-------------------|
| `?` | unversioned | `svn add` |
| `M` | modified | — |
| `A` | added | — |
| `D` | deleted | — |
| `!` | missing | `svn delete` |
| `C` | conflict | warn, exclude |

## Filter

Skip dirs: `dist/ build/ out/ node_modules/ __pycache__/ .cache/ coverage/ .pytest_cache/ .tox/ .mypy_cache/ .ruff_cache/ generated/ test-output/ test-results/ .eggs/`

Skip files: `*.log *.tmp *.pyc *.pyo *.generated.* *.auto.* *.min.js *.min.css *.map *.so *.dylib *.dll *.egg-info .env .env.* *.secret`

Split → **TO COMMIT** / **FILTERED OUT**.

## Confirm

Show both lists. Prompt: `Proceed? (y=commit / n=abort / e=edit list)`

All filtered → warn, offer override. No changes → "nothing to commit", stop.

## Msg

Diff target files (git: `git diff`; svn: `svn diff`).

**Git** — gitmoji fmt `emoji type: desc`:
- user hint given → combine intent + diff → gitmoji msg
- no hint → summarize diff → gitmoji msg

```
feat/new→✨  fix/bug→🐛  docs→📝  style/fmt→🎨
refactor→♻️  perf→⚡  test→✅  build/dep→📦
ci→👷  chore→🔧  remove→🔥  move→🚚
wip→🚧  security→🔒  init→🎉  hotfix→🚑
types→🏷️  breaking→💥
```

**SVN** — plain desc, no gitmoji:
- user hint given → combine intent + diff → desc msg, language matches user input
- no hint → summarize diff → English desc msg

Both: 50-72 chars, one line. Show proposed msg → user confirms/edits.

## Execute

**Git:**
```
git add <TO COMMIT files>
git commit -m "<msg>"
```

**SVN:**
```
svn add <? files>
svn delete <! files>
svn commit <all target files> -m "<msg>"
```

## Constraints

- CWD only. Never traverse other dirs.
- Commit only. No push.
- Never append Co-Authored-By.
- Conflict files (svn `C`) → warn, exclude always.
