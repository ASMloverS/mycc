---
name: doc-refine
description: >
  Compress docs. EN → caveman; CN → 文言文 (tech terms kept); other → caveman.
  `--english` flag → EN output. ONLY trigger on explicit `/doc-refine` command.
---

# doc-refine

Compress doc → essential form. Substance untouched.

## Params

| Param | Req | Default | Desc |
|-------|-----|---------|------|
| doc path/content | yes | — | File to refine |
| `--en` | no | false | → EN caveman output |
| `--lite`/`--full`/`--ultra` | no | ultra | Intensity |

## Rules

Preserve: facts, steps, warnings, IDs, code blocks, emphasis.
Keep tech terms exact: APIs, vars, cmds, paths, errors.
Doubt → keep.

## Workflow

```
1. Read src doc.
2. Pick route:
   a. --en set        → read references/style-en.md → EN caveman.
   b. src = CN        → read references/style-cn.md → 文言文.
   c. other           → caveman in that lang.
3. Rewrite per style guide.
4. Write back to src file.
5. Report: Updated <file>: <old> → <new> lines.
```

## Intensity

| Level | EN | CN |
|-------|----|----|
| lite | Drop filler, keep sentences | 白话精简，偶用文言 |
| full | Caveman: fragments, short syns | 全文文言文，四字格优先 |
| ultra | Abbrevs, arrows → causality | 极简文言，一字能尽不用两字 |

## Auto-Clarity

Suspend compression for: security warnings, irreversible confirmations, multi-step sequences where compression → misread risk. Resume after.

## Examples

`/doc-refine README.md` → CN detected → 文言文 per `references/style-cn.md`.

`/doc-refine docs/api.md --en --ultra` → `references/style-en.md` at ultra → EN caveman.

`/doc-refine docs/guide.md --lite` → detect lang → lite compression.
