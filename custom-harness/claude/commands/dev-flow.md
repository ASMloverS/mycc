---
allowed-tools: Read, Agent, Edit, Glob, Grep
description: Dev cycle — implement → review → fix → doc update. Reuses code-implementer + code-reviewer agents.
---

## Parse Input

`$ARGUMENTS` → detect:
- Ends `.md` + file exists → `spec_path` = path, `spec` = file content (Read it)
- Else → `spec` = `$ARGUMENTS` text, `spec_path` = none

## Step 1 — Implement

Bash `python ~/.claude/custom-harness/bin/dispatch.py code-implementer "Requirement/spec: <spec>\n\nImplement per your TDD workflow. Report: files changed, tests passing."` → parse JSON → Agent spawn.

Capture changed files from result.

## Step 2 — Review

Bash `python ~/.claude/custom-harness/bin/dispatch.py code-reviewer "Spec: <spec>\n\nImpl files: <changed files from step 1>\n\nReview for correctness, security, performance. Output per-finding severity (CRITICAL/MAJOR/MINOR/INFO) + summary table."` → parse JSON → Agent spawn.

## Step 3 — Fix Loop (max 2 iterations)

Parse review result:
- No CRITICAL/MAJOR → skip to step 4
- CRITICAL/MAJOR found + iteration < 2:
  1. Bash dispatch code-implementer: `"Original spec: <spec>\n\nFix all CRITICAL + MAJOR findings from review:\n<review findings>\n\nRun tests to verify all fixes pass."` → Agent spawn.
  2. Bash dispatch code-reviewer again w/ same spec + same impl files → Agent spawn.
  3. Increment iteration. Repeat check.

## Step 4 — Doc Status Update

Only if `spec_path` exists:
1. Glob sibling `tasks/` dir or same dir for `.md` files
2. Grep for `[ ]` checkboxes or `status:` fields matching task name/description
3. Edit: `[ ] ` → `[x] ` or `status: pending` → `status: done`
4. No match found → print "no task doc found, skipping status update"

## Output

Print final summary: impl status, review verdict, iterations used, doc update result.
