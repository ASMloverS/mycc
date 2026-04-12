---
allowed-tools: Read, Agent, Edit, Glob, Grep
description: Dev cycle — implement → review → fix → doc update. Reuses code-implementer + code-reviewer agents.
---

## Parse Input

`$ARGUMENTS` → detect:
- Ends `.md` + file exists → `spec_path` = path, `spec` = file content (Read it)
- Else → `spec` = `$ARGUMENTS` text, `spec_path` = none

## Step 1 — Implement

Spawn `Agent(subagent_type="code-implementer")`:

```
Requirement/spec:
<spec>

Implement per your TDD workflow. Report: files changed, tests passing.
```

Capture changed files from result.

## Step 2 — Review

Spawn `Agent(subagent_type="code-reviewer")`:

```
Spec:
<spec>

Impl files: <changed files from step 1>

Review for correctness, security, performance. Output per-finding severity (CRITICAL/MAJOR/MINOR/INFO) + summary table.
```

## Step 3 — Fix Loop (max 2 iterations)

Parse review result:
- No CRITICAL/MAJOR → skip to step 4
- CRITICAL/MAJOR found + iteration < 2:
  1. Spawn `Agent(subagent_type="code-implementer")`:
     ```
     Original spec:
     <spec>

     Fix all CRITICAL + MAJOR findings from review:
     <review findings>
     
     Run tests to verify all fixes pass.
     ```
  2. Spawn `Agent(subagent_type="code-reviewer")` again w/ same spec + same impl files
  3. Increment iteration. Repeat check.

## Step 4 — Doc Status Update

Only if `spec_path` exists:
1. Glob sibling `tasks/` dir or same dir for `.md` files
2. Grep for `[ ]` checkboxes or `status:` fields matching task name/description
3. Edit: `[ ] ` → `[x] ` or `status: pending` → `status: done`
4. No match found → print "no task doc found, skipping status update"

## Output

Print final summary: impl status, review verdict, iterations used, doc update result.
