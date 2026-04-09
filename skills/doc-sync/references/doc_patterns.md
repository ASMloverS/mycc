# Document Patterns

## Priority Order

1. Root docs (`README.md`, planning docs, testing docs).
2. `docs/**/*.md`.
3. Extra markdown files as needed.

## Common Drift Types

- Path drift after file moves.
- Feature drift after implementation changes.
- Missing mention of recently changed code files.
- Relative doc-local references that no longer resolve.
- Shorthand fixture paths that should be expanded to the actual repo path.

## Recommended Run Pattern

1. Run incremental check with `--from-ref`.
2. Apply only deterministic fixes.
3. Re-run check.
4. Handle unresolved findings manually.
5. Use compact output unless you are debugging a specific document.

## Review Checklist

- Do referenced files/directories exist?
- Are changed code files reflected in docs?
- Are unresolved findings truly stale or intentionally generic?
