---
name: doc-sync
description: Keep Markdown documentation aligned with real code and recent repository changes. Use when docs may be stale after refactors, feature delivery, file moves, command updates, or release prep. Works with any AI model and any toolchain by using repository facts, deterministic checks, and optional safe auto-fixes.
---

# Doc Sync

Synchronize docs with code as the source of truth.
Prefer deterministic checks and minimal edits.

## Core Tasks

1. Detect broken file or directory references in Markdown.
2. Detect changed code files that are not mentioned in docs.
3. Generate compact evidence-first reports.
4. Optionally apply safe path fixes with high confidence.

## Execution

Run with any Python runtime and any AI/tool wrapper:

```bash
python scripts/doc_sync.py --repo <repo-path> --mode check
python scripts/doc_sync.py --repo <repo-path> --mode check --from-ref <base-ref>
python scripts/doc_sync.py --repo <repo-path> --mode apply --from-ref <base-ref>
```

Default behavior: print the report to stdout only. Do not create a separate
report file unless the user explicitly asks for a persisted artifact.

## Modes

- `check`: analyze only.
- `apply`: apply deterministic path replacements only.

## Token/Cost Controls

- Default output is compact.
- Cap findings with `--max-findings`.
- Disable expensive checks unless needed.
- Prefer `--from-ref` for incremental runs.
- Prefer stdout over `--out` for normal skill execution.
- Resolve relative references against the document directory first.
- Ignore glob-style examples and placeholder paths unless they resolve to a real repo path.
- Prefer unique basename matches over broad fuzzy guesses.

## Arguments

- `--repo`: repository root.
- `--mode`: `check` or `apply`.
- `--from-ref`: optional git base ref for incremental coverage check.
- `--out`: optional report file; use only when a persistent artifact is explicitly requested.
- `--max-findings`: max emitted findings (default `120`).
- `--detail`: output verbose per-finding sections.
- `--ignore-doc`: repeatable glob for markdown files to skip.

## Safe-Edit Policy

- Preserve original doc structure and intent.
- Edit only exact backticked or linked path tokens.
- Treat relative doc-local references as valid when they resolve on disk.
- Skip glob patterns, placeholder paths, and ambiguous replacements.
- Report unresolved items for manual review.

## References

- Read `references/rules.md` for matching and confidence policy.
- Read `references/doc_patterns.md` for doc selection and review checklist.
