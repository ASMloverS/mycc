#!/usr/bin/env python3
"""Lightweight documentation sync checker.

Design goals:
- Model/tool agnostic.
- Conservative and deterministic.
- Compact report by default to reduce downstream token use.
"""

from __future__ import annotations

import argparse
import difflib
import fnmatch
import re
import subprocess
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

BACKTICK_TOKEN = re.compile(r"`([^`\n]+)`")
MARKDOWN_LINK = re.compile(r"\[[^\]]+\]\(([^)]+)\)")
SEGMENT_WORD = re.compile(r"^[A-Za-z0-9_-]+$")
COMPOSITE_SUFFIX = re.compile(r"/\.[A-Za-z0-9_-]+$")

GENERIC_PLACEHOLDER_SEGMENTS = {
    "foo",
    "bar",
    "baz",
    "qux",
    "xxx",
    "yyy",
    "zzz",
    "sample",
    "example",
    "examples",
}

LIKELY_CODE_EXTS = {
    "c",
    "cc",
    "cpp",
    "cxx",
    "h",
    "hh",
    "hpp",
    "py",
    "js",
    "ts",
    "tsx",
    "java",
    "rs",
    "go",
    "md",
    "txt",
    "json",
    "yaml",
    "yml",
    "toml",
    "cmake",
    "sh",
    "ps1",
}

DEFAULT_SKIP_DIRS = {
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    ".venv",
    "venv",
    "build",
    "dist",
    ".idea",
    ".vscode",
}

KNOWN_TOP_DIR_HINTS = {
    "src",
    "tests",
    "docs",
    "include",
    "scripts",
    "cmake",
    "python",
    "cpp",
    "assets",
    "examples",
}


@dataclass(frozen=True)
class Finding:
    severity: str
    kind: str
    file: Path
    message: str
    evidence: str


def run_git(repo: Path, args: list[str]) -> tuple[int, str]:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout.strip()


def should_skip_path(path: Path) -> bool:
    return any(part in DEFAULT_SKIP_DIRS for part in path.parts)


def list_repo_paths(repo: Path) -> tuple[set[str], set[str], set[str], dict[str, list[str]]]:
    files: set[str] = set()
    dirs: set[str] = set()
    roots: set[str] = set()
    basename_index: dict[str, list[str]] = defaultdict(list)

    for path in repo.rglob("*"):
        if should_skip_path(path):
            continue

        rel = path.relative_to(repo).as_posix()
        head = rel.split("/", 1)[0]
        if head:
            roots.add(head)

        if path.is_file():
            files.add(rel)
            basename_index[path.name.lower()].append(rel)
            parent = Path(rel).parent.as_posix()
            if parent != ".":
                dirs.add(parent)
        elif path.is_dir():
            dirs.add(rel)

    return files, dirs, roots, basename_index


def list_markdown_files(repo: Path, ignore_doc: list[str]) -> list[Path]:
    docs: list[Path] = []
    effective_ignore = list(ignore_doc) + ["docs-sync-report*.md"]

    for path in sorted(repo.rglob("*.md")):
        if should_skip_path(path):
            continue
        rel = path.relative_to(repo).as_posix()
        if any(fnmatch.fnmatch(rel, pat) for pat in effective_ignore):
            continue
        docs.append(path)

    return docs


def normalize_token(token: str) -> str:
    token = token.strip().strip("\"'")
    token = token.split("#", 1)[0].strip()
    token = token.replace("\\", "/")
    if token.startswith("./"):
        token = token[2:]
    return token


def token_is_url(token: str) -> bool:
    low = token.lower()
    return low.startswith(("http://", "https://", "mailto:", "file://"))


def token_is_absolute_path(token: str) -> bool:
    if re.match(r"^[A-Za-z]:/", token):
        return True
    if token.startswith("/"):
        return True
    return False


def token_is_pattern(token: str) -> bool:
    if not token:
        return False
    if any(ch in token for ch in "*?[]<>"):
        return True
    if "::" in token:
        return True
    if "@" in token:
        return True
    if COMPOSITE_SUFFIX.search(token):
        return True
    stem = token.rsplit("/", 1)[-1]
    if stem.split(".", 1)[0].lower() in GENERIC_PLACEHOLDER_SEGMENTS:
        return True
    return False


def slash_token_looks_like_path(token: str, repo_roots: set[str]) -> bool:
    segments = [s for s in token.split("/") if s]
    if not segments:
        return False

    if any(s.lower() in GENERIC_PLACEHOLDER_SEGMENTS for s in segments):
        if segments[0] not in repo_roots and segments[0] not in KNOWN_TOP_DIR_HINTS:
            return False

    first = segments[0]
    if first not in repo_roots and first not in KNOWN_TOP_DIR_HINTS and not first.startswith("."):
        # Avoid enum-like content such as this/super or if/while/for.
        if all(SEGMENT_WORD.match(s) for s in segments) and all("." not in s for s in segments):
            return False

    if all(s.isdigit() for s in segments):
        return False

    return True


def is_path_candidate(token: str, repo_roots: set[str]) -> bool:
    if not token or " " in token:
        return False
    if token in {"/", "\\"}:
        return False
    if token.startswith(".") and "/" not in token and token.count(".") == 1:
        # e.g. `.md`, `.cc`
        return False
    if token_is_url(token):
        return False
    if token_is_absolute_path(token):
        return False
    if token_is_pattern(token):
        return False

    if "/" in token:
        return slash_token_looks_like_path(token, repo_roots)

    if token in {"CMakeLists.txt", "Makefile", "Dockerfile"}:
        return True

    if token.endswith("*"):
        stem = token[:-1].rstrip("/")
        return "/" in stem

    if "." in token and not token.startswith("."):
        ext = token.rsplit(".", 1)[-1].lower()
        return ext in LIKELY_CODE_EXTS

    return False


def collect_doc_path_tokens(md_text: str, repo_roots: set[str]) -> set[str]:
    tokens: set[str] = set()

    for raw in BACKTICK_TOKEN.findall(md_text):
        norm = normalize_token(raw)
        if is_path_candidate(norm, repo_roots):
            tokens.add(norm.rstrip("/"))

    for raw in MARKDOWN_LINK.findall(md_text):
        norm = normalize_token(raw)
        if is_path_candidate(norm, repo_roots):
            tokens.add(norm.rstrip("/"))

    return tokens


def resolve_existing_reference(token: str, md: Path, repo: Path) -> str | None:
    for base in (md.parent, repo):
        candidate = base / token
        if not candidate.exists():
            continue
        try:
            return candidate.relative_to(repo).as_posix()
        except ValueError:
            continue

    return None


def changed_code_files(repo: Path, from_ref: str | None) -> list[str]:
    if not from_ref:
        return []

    code, out = run_git(repo, ["diff", "--name-only", f"{from_ref}..HEAD"])
    if code != 0 or not out:
        return []

    changed: list[str] = []
    for line in out.splitlines():
        rel = line.strip().replace("\\", "/")
        if not rel or rel.endswith(".md"):
            continue
        changed.append(rel)

    return changed


def make_replacement(old: str, candidates: set[str]) -> str | None:
    picked = difflib.get_close_matches(old, list(candidates), n=1, cutoff=0.90)
    return picked[0] if picked else None


def analyze_markdown(
    md_files: list[Path],
    repo: Path,
    repo_files: set[str],
    repo_dirs: set[str],
    repo_roots: set[str],
    basename_index: dict[str, list[str]],
    mode: str,
) -> tuple[list[Finding], int]:
    findings: list[Finding] = []
    updated = 0
    known = set(repo_files) | set(repo_dirs)

    for md in md_files:
        text = md.read_text(encoding="utf-8", errors="ignore")
        original = text
        tokens = collect_doc_path_tokens(text, repo_roots)

        for token in sorted(tokens):
            existing = resolve_existing_reference(token, md, repo)
            if existing and existing in known:
                continue

            basename_matches = basename_index.get(Path(token).name.lower(), [])
            replacement = None
            if len(basename_matches) == 1:
                replacement = basename_matches[0]
            else:
                replacement = make_replacement(token, known)

            if replacement:
                findings.append(
                    Finding(
                        severity="high",
                        kind="broken-path-reference",
                        file=md,
                        message=f"Missing `{token}`; candidate `{replacement}`.",
                        evidence="fuzzy path match >= 0.90",
                    )
                )
                if mode == "apply":
                    text = text.replace(f"`{token}`", f"`{replacement}`")
            else:
                findings.append(
                    Finding(
                        severity="high",
                        kind="broken-path-reference",
                        file=md,
                        message=f"Missing `{token}` with no high-confidence replacement.",
                        evidence="path not found in repository",
                    )
                )

        if mode == "apply" and text != original:
            md.write_text(text, encoding="utf-8", newline="\n")
            updated += 1

    return findings, updated


def analyze_coverage(repo: Path, md_files: list[Path], from_ref: str | None) -> list[Finding]:
    changed = changed_code_files(repo, from_ref)
    if not changed:
        return []

    blob = "\n".join(p.read_text(encoding="utf-8", errors="ignore").lower() for p in md_files)
    findings: list[Finding] = []
    for rel in changed:
        if rel.lower() not in blob:
            findings.append(
                Finding(
                    severity="medium",
                    kind="undocumented-change",
                    file=repo / rel,
                    message=f"Changed file `{rel}` is not referenced in docs.",
                    evidence=f"git diff {from_ref}..HEAD",
                )
            )

    return findings


def render_report(
    findings: list[Finding],
    mode: str,
    from_ref: str | None,
    updated: int,
    max_findings: int,
    detail: bool,
) -> str:
    counts = Counter((f.severity, f.kind) for f in findings)
    shown = findings[:max_findings]
    dropped = len(findings) - len(shown)

    lines: list[str] = [
        "# Doc Sync Report",
        "",
        f"- Mode: `{mode}`",
        f"- From ref: `{from_ref or 'N/A'}`",
        f"- Findings total: `{len(findings)}`",
        f"- Findings shown: `{len(shown)}`",
        f"- Files updated: `{updated}`",
    ]

    if counts:
        lines.append("- Counts:")
        for (severity, kind), num in sorted(counts.items()):
            lines.append(f"  - `{severity}/{kind}`: `{num}`")

    lines.append("")

    if not shown:
        lines.append("No issues detected.")
        return "\n".join(lines) + "\n"

    if detail:
        for idx, finding in enumerate(shown, start=1):
            lines.extend(
                [
                    f"## {idx}. {finding.kind}",
                    f"- Severity: `{finding.severity}`",
                    f"- File: `{finding.file.as_posix()}`",
                    f"- Message: {finding.message}",
                    f"- Evidence: {finding.evidence}",
                    "",
                ]
            )
    else:
        lines.append("## Findings (compact)")
        for idx, finding in enumerate(shown, start=1):
            lines.append(
                f"{idx}. [{finding.severity}] {finding.kind} | {finding.file.as_posix()} | {finding.message}"
            )

    if dropped > 0:
        lines.append("")
        lines.append(f"Additional findings not shown: `{dropped}` (increase `--max-findings`).")

    return "\n".join(lines) + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Sync Markdown docs with repository facts")
    parser.add_argument("--repo", type=Path, default=Path("."), help="Repository root")
    parser.add_argument("--mode", choices=["check", "apply"], default="check")
    parser.add_argument("--from-ref", default=None, help="Optional git base ref")
    parser.add_argument(
        "--out",
        type=Path,
        default=None,
        help="Optional report output path; omit for stdout-only runs",
    )
    parser.add_argument("--max-findings", type=int, default=120, help="Max findings in report")
    parser.add_argument("--detail", action="store_true", help="Emit verbose per-finding report")
    parser.add_argument("--ignore-doc", action="append", default=[], help="Glob for markdown files to skip")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo = args.repo.resolve()

    md_files = list_markdown_files(repo, args.ignore_doc)
    repo_files, repo_dirs, repo_roots, basename_index = list_repo_paths(repo)

    findings, updated = analyze_markdown(
        md_files,
        repo,
        repo_files,
        repo_dirs,
        repo_roots,
        basename_index,
        args.mode,
    )
    findings.extend(analyze_coverage(repo, md_files, args.from_ref))

    report = render_report(
        findings=findings,
        mode=args.mode,
        from_ref=args.from_ref,
        updated=updated,
        max_findings=max(1, args.max_findings),
        detail=args.detail,
    )

    if args.out:
        out = args.out if args.out.is_absolute() else repo / args.out
        out.write_text(report, encoding="utf-8", newline="\n")
    else:
        print(report)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
