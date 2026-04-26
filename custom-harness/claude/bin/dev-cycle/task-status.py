#!/usr/bin/env python3
"""task-status.py — update task status marker in TASKS.md.

Usage:
  python task-status.py --spec <spec.md> --to <done|in-progress|pending|cancelled>
                        [--tasks-index <path>] [--dry-run]

Exit codes:
  0  updated successfully
  2  TASKS index not found
  3  task already at target status (idempotent)
  4  ambiguous match or unknown marker style
  5  spec file missing
"""
import json
import os
import re
import sys
import tempfile
from pathlib import Path

_STATUS_EMOJI = {
    "pending":     "⬜",
    "in-progress": "🟨",
    "done":        "✅",
    "cancelled":   "❌",
}
_EMOJI_TO_STATUS = {v: k for k, v in _STATUS_EMOJI.items()}

_VALID_STATUSES = set(_STATUS_EMOJI.keys())


def _die(msg: str, code: int) -> None:
    print(json.dumps({"error": msg}), file=sys.stderr)
    sys.exit(code)


def _find_tasks_index(spec_path: Path) -> Path | None:
    """Walk upward from spec dir looking for TASKS.md or tasks.md."""
    d = spec_path.parent.resolve()
    for _ in range(10):
        for name in ("TASKS.md", "tasks.md"):
            candidate = d / name
            if candidate.exists():
                return candidate
        parent = d.parent
        if parent == d:
            break
        d = parent
    return None


def _extract_task_id(spec_path: Path, fm_text: str) -> str:
    # 1. frontmatter: task: T18
    m = re.search(r'^task:\s*([\w-]+)', fm_text, re.MULTILINE)
    if m:
        return m.group(1)
    # 2. filename regex T\d+
    m = re.search(r'(T\d+)', spec_path.stem)
    if m:
        return m.group(1)
    # 3. filename stem
    return spec_path.stem


def _parse_frontmatter(spec_path: Path) -> str:
    text = spec_path.read_text(encoding="utf-8")
    m = re.match(r'^---\s*\n(.*?)\n---', text, re.DOTALL)
    return m.group(1) if m else ""


def _replace_marker(line: str, target: str) -> str | None:
    """Return updated line or None if no known marker style found."""
    target_emoji = _STATUS_EMOJI[target]

    # emoji style
    for emoji in _EMOJI_TO_STATUS:
        if emoji in line:
            return line.replace(emoji, target_emoji, 1)

    # checkbox style (only pending↔done)
    if "[ ]" in line and target == "done":
        return line.replace("[ ]", "[x]", 1)
    if "[x]" in line and target == "pending":
        return line.replace("[x]", "[ ]", 1)
    if "[X]" in line and target == "pending":
        return line.replace("[X]", "[ ]", 1)

    # key-value style
    m = re.search(r'status:\s*\w+', line, re.IGNORECASE)
    if m:
        return line[:m.start()] + f"status: {target}" + line[m.end():]

    return None


def main() -> None:
    args = sys.argv[1:]

    spec_arg = tasks_arg = to_arg = None
    dry_run = False
    i = 0
    while i < len(args):
        if args[i] == "--spec" and i + 1 < len(args):
            spec_arg = args[i + 1]; i += 2
        elif args[i] == "--to" and i + 1 < len(args):
            to_arg = args[i + 1]; i += 2
        elif args[i] == "--tasks-index" and i + 1 < len(args):
            tasks_arg = args[i + 1]; i += 2
        elif args[i] == "--dry-run":
            dry_run = True; i += 1
        else:
            i += 1

    if not spec_arg:
        _die("--spec is required", 5)
    if not to_arg or to_arg not in _VALID_STATUSES:
        _die(f"--to must be one of: {', '.join(sorted(_VALID_STATUSES))}", 4)

    spec_path = Path(spec_arg)
    if not spec_path.exists():
        _die(f"spec file not found: {spec_arg}", 5)

    # Locate TASKS index
    if tasks_arg:
        tasks_path = Path(tasks_arg)
        if not tasks_path.exists():
            _die(f"tasks index not found: {tasks_arg}", 2)
    else:
        tasks_path = _find_tasks_index(spec_path)
        if tasks_path is None:
            _die("TASKS index not found (searched upward from spec dir)", 2)

    fm_text = _parse_frontmatter(spec_path)
    task_id = _extract_task_id(spec_path, fm_text)

    lines = tasks_path.read_text(encoding="utf-8").splitlines(keepends=True)

    # Find matching lines
    id_re = re.compile(re.escape(task_id))
    candidates = [(i, l) for i, l in enumerate(lines) if id_re.search(l)]

    if not candidates:
        _die(f"no line matching {task_id!r} in {tasks_path}", 4)

    # Prefer line with markdown link to spec
    spec_name = spec_path.name
    preferred = [(i, l) for i, l in candidates if spec_name in l]
    if len(preferred) == 1:
        candidates = preferred
    elif len(candidates) > 1:
        _die(
            f"ambiguous: {len(candidates)} lines match {task_id!r}. "
            f"Candidates: {[l.strip() for _, l in candidates]}",
            4,
        )

    line_idx, line = candidates[0]
    new_line = _replace_marker(line, to_arg)

    if new_line is None:
        _die(f"unknown marker style on line {line_idx + 1}: {line.strip()!r}", 4)

    if new_line == line:
        result = {"path": str(tasks_path), "line": line_idx + 1,
                  "before": line.strip(), "after": line.strip(), "status": "already_at_target"}
        print(json.dumps(result))
        sys.exit(3)

    result = {
        "path": str(tasks_path),
        "line": line_idx + 1,
        "before": line.strip(),
        "after": new_line.strip(),
    }
    print(json.dumps(result))

    if not dry_run:
        lines[line_idx] = new_line
        content = "".join(lines)
        fd, tmp = tempfile.mkstemp(dir=tasks_path.parent, suffix=".tmp")
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                f.write(content)
            os.replace(tmp, tasks_path)
        except Exception:
            os.unlink(tmp)
            raise


if __name__ == "__main__":
    main()
