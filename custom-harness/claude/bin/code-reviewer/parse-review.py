#!/usr/bin/env python3
"""parse-review.py — extract and validate REVIEW_RESULT block from reviewer output.

Usage:
  python parse-review.py [--file <path>]
  # reads from stdin if --file not given

Exit codes:
  0  verdict=pass
  1  verdict=fail
  5  block missing or malformed JSON
"""
import json
import re
import sys
from pathlib import Path

_BLOCK_RE = re.compile(
    r'<REVIEW_RESULT>\s*(\{.*?\})\s*</REVIEW_RESULT>',
    re.DOTALL,
)

_REQUIRED_KEYS = {"verdict", "crit", "maj", "min", "info", "findings"}


def _die(msg: str, code: int) -> None:
    print(json.dumps({"error": msg}), file=sys.stderr)
    sys.exit(code)


def main() -> None:
    args = sys.argv[1:]
    text: str

    if "--file" in args:
        idx = args.index("--file")
        if idx + 1 >= len(args):
            _die("--file requires a path argument", 5)
        path = args[idx + 1]
        if path == "/dev/stdin":
            text = sys.stdin.read()
        else:
            p = Path(path)
            if not p.exists():
                _die(f"file not found: {path}", 5)
            text = p.read_text(encoding="utf-8")
    else:
        text = sys.stdin.read()

    m = _BLOCK_RE.search(text)
    if not m:
        _die("no <REVIEW_RESULT> block found", 5)

    try:
        data = json.loads(m.group(1))
    except json.JSONDecodeError as e:
        _die(f"invalid JSON in block: {e}", 5)

    missing = _REQUIRED_KEYS - data.keys()
    if missing:
        _die(f"missing keys in REVIEW_RESULT: {missing}", 5)

    if data["verdict"] not in ("pass", "fail"):
        _die(f"verdict must be 'pass' or 'fail', got: {data['verdict']!r}", 5)

    print(json.dumps(data, ensure_ascii=False))

    if data["verdict"] == "pass":
        sys.exit(0)
    else:
        sys.exit(1)


if __name__ == "__main__":
    main()
