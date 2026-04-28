# User-Level CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

---

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
- Commit message format: `gitmoji type(scope): desc` — e.g. `✨ feat(auth): add login API`
- gitmoji mapping: feat/new→✨ fix→🐛 docs→📝 style→🎨 refactor→♻️ perf→⚡ test→✅ build→📦 ci→👷 chore→🔧 remove→🔥 wip→🚧
