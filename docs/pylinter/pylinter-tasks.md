# pylinter Task Index

Design: [pylinter-design.md](./pylinter-design.md)

## Phase 1: Scaffold + Core Formatting

| # | Task | Deps | Status |
|---|---|---|---|
| 01 | [Scaffold — Cargo.toml + main.rs + lib.rs](./task-01-scaffold.md) | — | ✅ Done |
| 02 | [Config — config.rs + .pylinter.yaml](./task-02-config.md) | 01 | ✅ Done |
| 03 | [Common — SourceFile + Diagnostic + string_utils](./task-03-common.md) | 01 | ⬜ TODO |
| 04 | [CLI + file collect + ignore](./task-04-cli.md) | 02, 03 | ⬜ TODO |
| 05 | [CST core — tokenize + parse + CSTSource + regenerate](./task-05-cst-core.md) | 03 | ⬜ TODO |
| 06 | [Formatter — encoding (UTF-8 + LF + BOM)](./task-06-encoding.md) | 05 | ⬜ TODO |
| 07 | [Formatter — trailing whitespace removal](./task-07-trailing-ws.md) | 05 | ⬜ TODO |
| 08 | [Formatter — indent](./task-08-indent.md) | 05 | ⬜ TODO |
| 09 | [Formatter — blank_lines normalize](./task-09-blank-lines.md) | 05, 08 | ⬜ TODO |
| 10 | [Formatter pipeline + E2E tests](./task-10-formatter-pipeline.md) | 06~09 | ⬜ TODO |

## Phase 2: Advanced Formatting + All Checkers

| # | Task | Deps | Status |
|---|---|---|---|
| 11 | [Formatter — import_sort (isort-style)](./task-11-import-sort.md) | 05, 10 | ⬜ TODO |
| 12 | [Formatter — comment_style unify](./task-12-comment-style.md) | 05 | ⬜ TODO |
| 13 | [Formatter — line_length + wrap](./task-13-line-length.md) | 05 | ⬜ TODO |
| 14 | [Formatter — binary_op line break](./task-14-binary-op.md) | 05 | ⬜ TODO |
| 15 | [Checker — naming](./task-15-checker-naming.md) | 05 | ⬜ TODO |
| 16 | [Checker — complexity](./task-16-checker-complexity.md) | 05 | ⬜ TODO |
| 17 | [Checker — magic_number](./task-17-checker-magic-number.md) | 05 | ⬜ TODO |
| 18 | [Checker — unused_import](./task-18-checker-unused-import.md) | 05 | ⬜ TODO |
| 19 | [Checker — prohibited fn/modules](./task-19-checker-prohibited.md) | 05 | ⬜ TODO |
| 20 | [Checker — docstring](./task-20-checker-docstring.md) | 05 | ⬜ TODO |
| 21 | [Checker pipeline + Phase 2 formatter wrap-up](./task-21-checker-pipeline.md) | 10~20 | ⬜ TODO |

## Phase 3: Analyzers

| # | Task | Deps | Status |
|---|---|---|---|
| 22 | [Analyzer — basic (common pitfalls)](./task-22-analyzer-basic.md) | 05 | ⬜ TODO |
| 23 | [Analyzer — strict (code quality)](./task-23-analyzer-strict.md) | 05 | ⬜ TODO |
| 24 | [Analyzer — deep](./task-24-analyzer-deep.md) | 05 | ⬜ TODO |
| 25 | [Analyzer pipeline + full E2E tests](./task-25-analyzer-pipeline.md) | 21~24 | ⬜ TODO |

## Dependency Graph

```
Phase 1:
  01 ──┬──> 02 ──┐
       │         v
       ├──> 03 ──> 04
       │         
       └──> 05 ──┬──> 06 ─┐
                  ├──> 07 ─┤
                  ├──> 08 ─┼──> 09 ──┐
                  └───────┴─────────┴──> 10

Phase 2:
  05 ──┬──> 11 ─┐
       ├──> 12 ─┤
       ├──> 13 ─┼──> 21
       ├──> 14 ─┤    ^
       ├──> 15 ─┤    |
       ├──> 16 ─┤    10
       ├──> 17 ─┤
       ├──> 18 ─┤
       ├──> 19 ─┤
       └──> 20 ─┘

Phase 3:
  05 ──┬──> 22 ─┐
       ├──> 23 ─┼──> 25
       └──> 24 ─┘    ^
                      21
```
