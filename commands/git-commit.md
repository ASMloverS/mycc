---
allowed-tools: Bash(git status:*), Bash(git branch:*), Bash(git log:*), Agent
description: Git add+commit w/ smart filtering & gitmoji. Spawns haiku agent.
---

## Parse Args

`$ARGUMENTS` → extract:
- `dir`: first non-flag token (default: CWD)
- `--push`: push after commit
- `--include=P`: comma-sep patterns to force-include
- `--exclude=P`: comma-sep patterns to extra-exclude

## Gather Context

Run in target dir:
- `!git status --porcelain`
- `!git branch --show-current`
- `!git log --oneline -5`

## Spawn Agent

Pass all context to `git-committer` agent:

```
Target dir: <resolved dir>
Flags: --push=<bool> --include=<P> --exclude=<P>
Branch: <branch>
Recent commits: <log>
Git status:
<status output>
```

Spawn agent w/ subagent_type `git-committer`. Agent handles filtering, confirmation, msg generation, commit, optional push.
