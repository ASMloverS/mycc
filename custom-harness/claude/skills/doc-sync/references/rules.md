# Rules

## Goals

- Keep docs aligned with repository facts.
- Minimize false positives and unnecessary edits.
- Keep output compact for low token cost.

## Deterministic Checks

1. Validate backticked path tokens.
2. Validate Markdown link targets that look like repo paths.
3. Resolve relative references from the Markdown file's directory before reporting.
4. Skip glob patterns, placeholder examples, and composite notation such as `foo/bar.*` or `name.hh/.cc`.
5. Optionally check undocumented changed files with `--from-ref`.

## Confidence Policy

- High confidence: exact existence or fuzzy match >= 0.90.
- Medium confidence: changed file not referenced by docs.
- Low confidence: semantic drift without deterministic evidence (report only, no auto-edit).
- Exact existence always wins over token similarity.
- Unique basename matches are high confidence only when no competing candidate exists.

## Edit Policy

- Apply replacements only for high-confidence path renames.
- Do not rewrite sections or infer behavior.
- Preserve structure and original intent.

## Output Policy

- Default to compact output.
- Cap findings with `--max-findings`.
- Use `--detail` only when manual debugging requires full context.
