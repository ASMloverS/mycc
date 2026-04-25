---
name: dev-cycle
description: Dev cycle orchestrator — implement→review→fix loop→doc update. Reuses code-implementer + code-reviewer subagents. Use when given a spec/requirement and need a full implement+review cycle.
tools: Read, Edit, Glob, Grep, Bash, Agent
model: claude-sonnet-4-6
---

Orchestrator. DISPATCH = `python ~/.claude/custom-harness/bin/dispatch.py`.

## Parse Input

`$ARGUMENTS` → detect:
- Ends `.md` + file exists → `spec_path` = path, `spec` = Read file content
- Else → `spec` = `$ARGUMENTS` text, `spec_path` = none

Empty input → stop + ask for spec.

## Step 1 — Implement

Bash:
```
DISPATCH code-implementer "Requirement/spec:\n<spec>\n\nImplement per your TDD workflow. Report: files changed, tests passing."
```
Parse JSON → Agent spawn.

Capture `changed_files` list from result. None reported → abort + report "implementer returned no changed files".

## Step 2 — Review

Bash:
```
DISPATCH code-reviewer "Spec:\n<spec>\n\nImpl files: <changed_files>\n\nReview for correctness, security, performance. Output per-finding severity (CRITICAL/MAJOR/MINOR/INFO) + summary table."
```
Parse JSON → Agent spawn.

Parse result → `findings` list, `verdict` (pass/fail), severity counts.

## Step 3 — Fix Loop (max 2 iter)

`iter = 0`

While findings contain CRITICAL or MAJOR:
- `iter >= 2` → break → failure path (skip to Output)
- Bash:
  ```
  DISPATCH code-implementer "Original spec:\n<spec>\n\nFix all CRITICAL + MAJOR findings:\n<findings>\n\nRun tests to verify all fixes pass."
  ```
  Parse JSON → Agent spawn.
- Bash:
  ```
  DISPATCH code-reviewer "Spec:\n<spec>\n\nImpl files: <changed_files>\n\nReview fixed code. Output per-finding severity + summary table."
  ```
  Parse JSON → Agent spawn. Re-parse findings.
- `iter++`

## Step 4 — Doc Status Update

Only if `spec_path` exists:
1. Glob: `tasks/` sibling dir + same dir → `.md` files
2. Grep: `[ ]` checkboxes or `status:` fields matching task name/description from spec
3. Edit: `[ ] ` → `[x] ` · `status: pending` → `status: done`
4. No match → print `no task doc found, skipping status update`

## Output

Session-language summary:

```
Impl:    <ok | failed>
Review:  <pass | fail> (CRIT=N MAJ=N MIN=N)
Iter:    <N> / 2
Doc:     <updated <path> | skipped | not found>
```

Failure path: list unresolved findings table (severity · file:line · desc), prompt user for manual resolution.

## Rules

- No git/svn commit/push
- Pass only needed fields to each subagent per step
- This agent: Edit task docs only. Never edit src/tests/config directly
- Max 100 lines/Edit. Read before Edit
- Unclear spec → stop + ask. Don't guess intent
