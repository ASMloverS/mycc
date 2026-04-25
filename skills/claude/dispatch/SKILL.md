---
name: dispatch
description: User-invoked router. Spawn registered agent/command/skill as one-shot subagent. Trigger ONLY on explicit /dispatch from user.
---

# dispatch

## Invoke

User: `/dispatch <name|type:name> [<prompt>]` [`--model M`] [`--bg`] [`--help [name]`]

If `<prompt>` is omitted, the agent receives an empty `User Input` block — agents that document defaults (like `vsc-committer`) will run with those defaults.

Parallel: `/dispatch --parallel "name1 prompt1" "name2 prompt2" …` [`--model M`] [`--bg`]
Each quoted token: first word = name, rest = prompt.

## Flow

1. Bash → `python ~/.claude/custom-harness/bin/dispatch.py <argv>`
2. Parse stdout JSON
3. **If array**: call Agent tool N times **in a single message** (parallel spawn). Collect all results.
   **If object**: call Agent tool once. Return result.
4. Return subagent result(s)

## `--help`

Forward to dispatch.py. Print stdout verbatim. No subagent spawn.

## Errors

Non-zero exit → print stderr. No retry.

## NEVER

- Don't guess names. Ambiguity → ask user.
- Don't trigger without explicit /dispatch.
- Don't inline registry lookup.
