---
name: dispatch
description: User-invoked router. Spawn registered agent/command/skill as one-shot subagent. Trigger ONLY on explicit /dispatch from user.
---

# dispatch

## Invoke

User: `/dispatch <name|type:name> <prompt>` [`--model M`] [`--bg`] [`--help [name]`]

## Flow

1. Bash → `python ~/.claude/custom-harness/bin/dispatch.py <argv>`
2. Parse stdout JSON
3. Call Agent tool w/ JSON fields
4. Return subagent result

## `--help`

Forward to dispatch.py. Print stdout verbatim. No subagent spawn.

## Errors

Non-zero exit → print stderr. No retry.

## NEVER

- Don't guess names. Ambiguity → ask user.
- Don't trigger without explicit /dispatch.
- Don't inline registry lookup.
