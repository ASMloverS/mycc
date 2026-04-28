---
name: dispatch
description: User-invoked router. Spawn registered agent/command/skill as one-shot subagent. Trigger ONLY on explicit /dispatch from user.
---

# dispatch

## Invoke

User: `/dispatch <name|type:name> [<prompt>]` [`--model M`] [`--bg`] [`--inline`] [`--help [name]`]

If `<prompt>` is omitted, the agent receives an empty `User Input` block — agents that document defaults (like `vsc-committer`) will run with those defaults.

`--inline`: embed the full MD body directly in the subagent prompt instead of using a thin pointer. Use for debugging or behavior comparison only; default is thin-pointer mode.

Parallel: `/dispatch --parallel "name1 prompt1" "name2 prompt2" …` [`--model M`] [`--bg`]
Each quoted token: first word = name, rest = prompt.

## Flow

1. Bash → `~/.claude/custom-harness/bin/dispatch <argv>`
2. Parse stdout JSON
3. Read envelope `mode` field:
   - `mode="single"` → call Agent tool once with `payloads[0]`
   - `mode="parallel"` → in **one message**, call Agent tool once per item in `payloads`
4. Subagent behavior (thin-pointer mode, default):
   - Receives a wrapper prompt containing `DEFINITION_FILE`, `HARNESS_DIR`, `TYPE`, `TOOL_HINT`, and the user input
   - **First action**: `Read DEFINITION_FILE` to load its full instructions into its own context
   - With `--inline`, the full MD body is embedded directly in the prompt instead (no `Read` needed)
5. Return subagent result(s)

## `--help`

Forward to dispatch. Print stdout verbatim. No subagent spawn.

## Errors

Non-zero exit → print stderr. No retry.

## NEVER

- Don't guess names. Ambiguity → ask user.
- Don't trigger without explicit /dispatch.
- Don't inline registry lookup.
