---
description: Git add+commit w/ smart filtering & gitmoji
agent: git-committer
model: zai-coding-plan/glm-4.5-air
subtask: true
---

Run pre-check script:

`!python ~/.config/opencode/commands/tools/git_commit_precheck.py $ARGUMENTS`

- If output contains `STATUS=CLEAN` → print "Nothing to commit." and stop.
- Otherwise → spawn `git-committer` agent, pass the full script output verbatim as context.
