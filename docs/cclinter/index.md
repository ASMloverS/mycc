# cclinter Implementation Task Index

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement tasks. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust-based C language linter with formatting, style checking, and static analysis.

**Architecture:** Regex/text-matching parser. Three engines (formatter → checker → analyzer) driven by clap CLI. YAML config (`serde_yaml`). Parallel file processing via rayon.

**Tech Stack:** Rust stable, clap, serde_yaml, regex, rayon, walkdir, globset, colored, similar, tempfile (dev)

**Design Doc:** `docs/cclinter/cclinter-design.md`

**Tool Location:** `tools/linter/cclinter/`

---

## Phase 1 — Formatting

| Task | File | Description | Status |
|------|------|-------------|--------|
| T01 | `phase1-T01-skeleton.md` | Project skeleton, Cargo.toml, clap CLI, module stubs | ✅ Done |
| T02 | `phase1-T02-encoding.md` | UTF-8 BOM removal, CRLF→LF, trailing whitespace strip | ✅ Done |
| T03 | `phase1-T03-indent.md` | Tab→2-space, brace-level indentation | ✅ Done |
| T04 | `phase1-T04-spacing.md` | Operator/comma/paren/semicolon spacing rules | ✅ Done |
| T05 | `phase1-T05-braces.md` | Brace attach style (Google) | ✅ Done |
| T06 | `phase1-T06-blank-lines.md` | Blank line normalization | ✅ Done |
| T07 | `phase1-T07-comments.md` | `/* */` → `//` conversion (all, including copyright) | ✅ Done |
| T08 | `phase1-T08-pointer-style.md` | Pointer alignment: `int *p` → `int* p` | ✅ Done |
| T09 | `phase1-T09-switch-indent.md` | switch-case indentation | ✅ Done |
| T10 | `phase1-T10-alignment.md` | Continuation alignment + struct/enum alignment | ✅ Done |
| T11 | `phase1-T11-line-length.md` | 120-column line wrapping | ✅ Done |
| T12 | `phase1-T12-include-sort.md` | #include Google three-group sorting | ✅ Done |
| T13 | `phase1-T13-config.md` | YAML config loading + directory lookup | ✅ Done |
| T14 | `phase1-T14-ignore.md` | `.cclinterignore` support | ✅ Done |
| T15 | `phase1-T15-modes.md` | `--diff` / `--check` / `-i` modes | ✅ Done |
| T16 | `phase1-T16-parallel.md` | rayon parallel file processing | ✅ Done |
| T17 | `phase1-T17-tests.md` | Unit tests + snapshot test framework | ⬜ Stub |

## Phase 2 — Style Checking

| Task | File | Description | Status |
|------|------|-------------|--------|
| T01 | `phase2-T01-diag-framework.md` | clang-tidy diagnostic output + rule trait | ✅ Done |
| T02 | `phase2-T02-naming.md` | Naming convention checks | ⬜ Stub |
| T03 | `phase2-T03-include-guard.md` | Include guard + duplicate include detection | ⬜ Stub |
| T04 | `phase2-T04-complexity.md` | Function/file line count, nesting depth | ⬜ Stub |
| T05 | `phase2-T05-magic-number.md` | Magic number detection + allowlist | ⬜ Stub |
| T06 | `phase2-T06-unused.md` | Unused variable/macro/param detection | ⬜ Stub |
| T07 | `phase2-T07-prohibited.md` | Prohibited function check + YAML extend/remove | ⬜ Stub |
| T08 | `phase2-T08-forward-decl.md` | Forward declaration check | ⬜ Stub |
| T09 | `phase2-T09-integration.md` | Exit code 2, checker integration, tests | ⬜ Stub |

## Phase 3 — Static Analysis

| Task | File | Description | Status |
|------|------|-------------|--------|
| T01 | `phase3-T01-level-framework.md` | Analysis level framework (basic/strict/deep) | ✅ Done |
| T02 | `phase3-T02-basic.md` | basic level rules | ⬜ Stub |
| T03 | `phase3-T03-strict.md` | strict level rules | ⬜ Stub |
| T04 | `phase3-T04-deep.md` | deep level rules | ⬜ Stub |
| T05 | `phase3-T05-integration.md` | Exit code 4, full integration + cross-platform tests | ⬜ Stub |

---

## Dependency Graph

```
Phase 1 (sequential within, T01 must be first, T13 should precede T14-T16):
  T01 → T02 → T03 → T04 → T05 → T06 → T07 → T08 → T09 → T10 → T11 → T12
  T13 → T14, T15, T16
  T17 (after all formatting tasks)

Phase 2 (T01 must be first, others independent):
  T01 → T02..T08 (parallel)
  T09 (after all checker tasks)

Phase 3 (T01 must be first, T02..T04 parallel):
  T01 → T02..T04 (parallel)
  T05 (after all)
```
