# Repo Rules

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
