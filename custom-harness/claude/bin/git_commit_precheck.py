#!/usr/bin/env python3
"""git-commit pre-check: parse args + check for changes."""
from __future__ import annotations

import os
import subprocess
import sys


def parse_args(argv: list[str]) -> tuple[str, bool, str, str]:
    """Parse CLI arguments into (dir, push, include, exclude)."""
    push: bool = False
    include: str = ""
    exclude: str = ""
    positional: list[str] = []

    for arg in argv:
        if arg == "--push":
            push = True
        elif arg.startswith("--include="):
            include = arg.split("=", 1)[1]
        elif arg.startswith("--exclude="):
            exclude = arg.split("=", 1)[1]
        else:
            positional.append(arg)

    dir_arg: str = os.path.abspath(positional[0] if positional else ".")
    return dir_arg, push, include, exclude


def get_git_status(cwd: str) -> str:
    """Run git status --porcelain and return stripped output."""
    result: subprocess.CompletedProcess[str] = subprocess.run(
        ["git", "status", "--porcelain"],
        capture_output=True, text=True, cwd=cwd,
    )
    return result.stdout.strip()


def main() -> None:
    dir_arg, push, include, exclude = parse_args(sys.argv[1:])
    status: str = get_git_status(dir_arg)

    if not status:
        print("STATUS=CLEAN")
        sys.exit(0)

    print("STATUS=DIRTY")
    print(f"DIR={dir_arg}")
    print(f"PUSH={str(push).lower()}")
    if include:
        print(f"INCLUDE={include}")
    if exclude:
        print(f"EXCLUDE={exclude}")
    print("---GIT_STATUS---")
    print(status)


if __name__ == "__main__":
    main()
