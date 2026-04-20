---
allowed-tools: Bash(python*), Agent
description: Git add+commit w/ smart filtering & gitmoji. Spawns haiku agent.
---

Run pre-check script:

`!python ~/.claude/custom-harness/bin/git_commit_precheck.py $ARGUMENTS`

- If output contains `STATUS=CLEAN` → print "Nothing to commit." and stop.
- Otherwise → spawn `git-committer` agent, pass the full script output verbatim as context.
