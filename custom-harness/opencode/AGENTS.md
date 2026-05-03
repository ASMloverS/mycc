# Repo Rules

## Behavioral Guidelines

### Think Before Coding
- State assumptions explicitly. Uncertain → ask.
- Multiple interpretations exist → present all, don't pick silently.
- Simpler approach exists → say so. Push back when warranted.
- Unclear → stop. Name what's confusing. Ask.

### Simplicity First
- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

### Surgical Changes
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- Unrelated dead code → mention it, don't delete it.
- Remove imports/variables/functions YOUR changes made unused.
- Every changed line must trace directly to the user's request.

### Goal-Driven Execution
- Transform tasks into verifiable goals:
  - "Add validation" → "Write tests for invalid inputs, make them pass"
  - "Fix the bug" → "Write a test that reproduces it, make it pass"
  - "Refactor X" → "Ensure tests pass before and after"
- Multi-step tasks → state a brief plan with verify checkpoints.

## Edit Protocol

### Read
- Grep target symbol first → Read ±20 lines offset/limit (≤300 lines).
- Full-file read only when survey needed.

### Write: 100-Line Rule
- ≤100 lines per Edit/Write call.
- Larger → split: Edit-Verify cycle (sub-change → build check → next).
- 1000+ line rename/refactor → `.patch` or `sed`.

### Forbidden
- No multi-fn refactor in single Edit.
- No overwrite without Grep/Read first.

## File Hygiene
- UTF-8, LF endings.
- No trailing whitespace.
