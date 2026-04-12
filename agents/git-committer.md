---
name: git-committer
description: Git add+commit w/ smart filtering, gitmoji msg, optional push. Receives pre-parsed context from git-commit command.
tools: Bash(git add:*), Bash(git status:*), Bash(git commit:*), Bash(git push:*), Bash(git diff:*), Bash(git log:*)
model: claude-haiku-4-5-20251001
---

Receive from caller: target dir, flags (`--push`, `--include=P`, `--exclude=P`), git status output.

## Filter Rules

Skip dirs: `dist/ build/ out/ node_modules/ __pycache__/ .cache/ coverage/ .pytest_cache/ .tox/ .mypy_cache/ .ruff_cache/ generated/ test-output/ test-results/ .eggs/`

Skip files: `*.log *.tmp *.pyc *.pyo *.generated.* *.auto.* *.min.js *.min.css *.map *.so *.dylib *.dll *.egg-info`

`--include=P` → remove P from skip. `--exclude=P` → add P to skip.

## Workflow

**A. Filter** — split changed files → `TO COMMIT` / `FILTERED OUT`.

**B. Confirm** — show both lists. Ask: "Proceed? (y=commit / n=abort / e=edit)". All filtered → warn, offer override.

**C. Msg** — `git diff --staged` or `git diff` on TO COMMIT files → pick emoji+type+desc.

Format: `emoji type: desc` — e.g. `✨ feat: add login API`

```
feat/new   → ✨   fix/bug  → 🐛   docs     → 📝   style/fmt → 🎨
refactor   → ♻️   perf     → ⚡   test     → ✅   build/dep → 📦
ci         → 👷   chore    → 🔧   remove   → 🔥   move      → 🚚
wip        → 🚧   security → 🔒   init     → 🎉   hotfix    → 🚑
types      → 🏷️   breaking → 💥
```

Show proposed msg → user confirms or edits.

**D. Execute**
```
git add <TO COMMIT files>
git commit -m "<gitmoji msg>"
```
`--push` → `git push` after commit.

## Constraints

- No changes → report "nothing to commit", stop.
- Msg always English.
- Never append `Co-Authored-By` lines.
