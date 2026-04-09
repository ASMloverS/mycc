---
name: doc-write
description: >
  Write docs from context. CN → 文言文 ultra; `--en`/`--english` → EN caveman ultra.
  Gathers info from conversation history, source code, project structure, existing docs.
  Smart merge by section on existing files. ONLY trigger on explicit `/doc-write` command.
---

# doc-write

Write docs from context in ultra-compressed style. Markdown only.

## Params

| Param | Req | Default | Desc |
|-------|-----|---------|------|
| target path | no | auto in cwd | File/dir to write |
| `--en` / `--english` | no | false | EN caveman output |

Command: `/doc-write [path] [--en]` — param order free.

## Rules

- Fixed ultra compression. No intensity param.
- Write in ultra style directly. Do NOT write normal then compress.
- Format: Markdown only.
- Preserve: facts, steps, warnings, IDs, code blocks, emphasis.
- Keep tech terms exact: APIs, vars, cmds, paths, errors.
- Doubt → keep.
- Suspend compression for: security warnings, irreversible confirmations, multi-step sequences where compression → misread risk.
- Terminology/naming: reference project's existing docs for consistency.

## Workflow

```
1. Parse /doc-write command. Extract target path + --en flag.
2. Gather context:
   a. Review conversation history for relevant info.
   b. Scan project directory structure.
   c. Read source code files mentioned or relevant.
   d. Read existing docs (README etc.) for terminology reference.
3. Read references/ultra-rules.md for compression rules.
4. Pick style:
   a. --en set → EN caveman ultra (per ultra-rules.md EN section).
   b. default → CN 文言文 ultra (per ultra-rules.md CN section).
5. Determine target:
   a. Path given + is file → use that file.
   b. Path given + is dir → auto-name in that dir.
   c. No path → auto-name in cwd.
   Auto-name rule: derive from module/class/topic. e.g. "API docs for auth module" → auth-api.md.
6. Check target file existence:
   a. Not exists → create new.
   b. Exists → smart merge:
      - Parse both old and new by Markdown headings (## / ###).
      - Same heading → replace section content.
      - New heading → append to end.
      - Preserve content in old that has no matching new section.
7. Write result to target file.
8. Report summary.
```

## Smart Merge

Section matching by heading text (case-insensitive, strip whitespace).

```
Old file:              New content:
## Install             ## Install
old install steps      new install steps
## Usage               ## Config
old usage              new config section
## License
MIT

Merged result:
## Install
new install steps
## Usage
old usage
## Config
new config section
## License
MIT
```

Top-level content (before first heading): new replaces old.

## Report Format

After writing, output:

```
📄 doc-write report:
  File: <path>
  Status: created | updated
  Lines: <old_lines> → <new_lines> (for updates)
  Sections:
    + <new section names>
    ~ <replaced section names>
    = <unchanged section names>
```

For created files, omit unchanged/sections details, show full section list.

## Examples

`/doc-write docs/api.md`
→ Gather context → detect CN → 文言文 ultra → create docs/api.md.

`/doc-write --en`
→ Gather context → EN caveman ultra → auto-name in cwd → create file.

`/doc-write README.md --english`
→ Gather context → EN caveman ultra → merge into existing README.md by sections.
