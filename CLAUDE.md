# User-Level CLAUDE.md

## Precision Editing Protocol

### Read: Locate-Window-Verify
- Grep target first → Read with offset/limit (max 300 lines).
- Never read from line 1 unless full survey needed.
- Include ±20 lines around target before editing.

### Write: 100-Line Rule
- Max **100 lines** per Edit/Write.
- Larger changes → **Edit-Verify** cycle:
  1. Sub-change (≤100 lines).
  2. Syntax/build check (`cmake --build build`, `g++ -fsyntax-only`).
  3. Repeat.
- 1000+ line renames → `.patch` or `sed`, not Edit.

### Forbidden
- No mega-edits: multiple fns in one Edit call.
- No blind overwrites: Grep/Read before writing.

## Git Commit Convention
- Never append `Co-Authored-By: Claude ...` to commit messages.
# graphify
- **graphify** (`~/.claude/skills/graphify/SKILL.md`) - any input to knowledge graph. Trigger: `/graphify`
When the user types `/graphify`, invoke the Skill tool with `skill: "graphify"` before doing anything else.
