#!/usr/bin/env python3
"""vsc-commit.py — VCS add+commit. Usage: [DIR] -m MSG [--push] [--svn] [--dry-run] [--include=P] [--exclude=P]"""
import fnmatch, subprocess, sys
from pathlib import Path

SKIP_DIRS = {"dist","build","out","node_modules","__pycache__",".cache","coverage",".pytest_cache",
             ".tox",".mypy_cache",".ruff_cache","generated","test-output","test-results",".eggs"}
SKIP_PATS  = ["*.log","*.tmp","*.pyc","*.pyo","*.generated.*","*.auto.*","*.min.js","*.min.css",
              "*.map","*.so","*.dylib","*.dll","*.egg-info",".env",".env.*","*.secret"]

def die(msg, code=1):
    print(f"vsc-commit: {msg}", file=sys.stderr); sys.exit(code)

def run(cmd, cwd, dry=False):
    print(f"+ {' '.join(str(c) for c in cmd)}")
    if dry: return ""
    r = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True)
    if r.stdout: print(r.stdout, end="")
    if r.returncode != 0: print(r.stderr, file=sys.stderr); die(f"failed: {cmd[0]}", 4)
    return r.stdout

def query(cmd, cwd):
    return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True).stdout

def parse_args(argv):
    args, msg, push, svn_f, dry, inc, exc, d = argv[1:], None, False, False, False, [], [], "."
    i = 0
    while i < len(args):
        a = args[i]
        if   a == "-m" and i+1 < len(args):  msg = args[i+1]; i += 2
        elif len(a) > 2 and a[:2] == "-m":   msg = a[2:]; i += 1
        elif a == "--push":                   push = True; i += 1
        elif a == "--svn":                    svn_f = True; i += 1
        elif a == "--dry-run":                dry = True; i += 1
        elif a.startswith("--include="):      inc.append(a[10:]); i += 1
        elif a.startswith("--exclude="):      exc.append(a[10:]); i += 1
        elif not a.startswith("-"):           d = a; i += 1
        else: die(f"unknown: {a}", 2)
    if not msg: die("missing -m MESSAGE", 2)
    return dict(dir=d, msg=msg, push=push, svn=svn_f, dry=dry, inc=inc, exc=exc)

def detect_vcs(d, svn_f):
    if svn_f:                return "svn"
    if (d/".git").exists():  return "git"
    if (d/".svn").exists():  return "svn"
    die(f"no repository found in {d}", 3)

def should_skip(path, inc, exc):
    parts = Path(path).parts
    dirs = (SKIP_DIRS - set(inc)) | set(exc)
    if any(p in dirs for p in parts[:-1]): return True
    name = parts[-1]
    return (any(fnmatch.fnmatch(name, p) for p in SKIP_PATS if p not in inc) or
            any(fnmatch.fnmatch(name, p) for p in exc))

def git_changes(cwd):
    result = []
    for line in query(["git","status","--porcelain"], cwd).splitlines():
        if len(line) < 3: continue
        path = line[3:].strip()
        if " -> " in path: path = path.split(" -> ")[1]
        result.append((line[:2].strip(), path))
    return result

def svn_changes(cwd):
    return [(l[0], l[8:].strip()) for l in query(["svn","status"], cwd).splitlines() if l.strip()]

def apply_filter(changes, inc, exc):
    keep, dropped = [], []
    for item in changes: (dropped if should_skip(item[1], inc, exc) else keep).append(item)
    return keep, dropped

def do_git(opts, cwd, keep):
    dry = opts["dry"]
    files = [p for _, p in keep]
    run(["git", "add"] + files, cwd, dry)
    run(["git", "commit", "-m", opts["msg"]], cwd, dry)
    if opts["push"]: run(["git", "push"], cwd, dry)
    if not dry: print(query(["git", "status"], cwd))

def do_svn(opts, cwd, keep):
    dry = opts["dry"]
    conflicts = [p for st, p in keep if st == "C"]
    if conflicts:
        print(f"WARNING: {len(conflicts)} conflict(s) excluded: {conflicts}", file=sys.stderr)
        keep = [(st, p) for st, p in keep if st != "C"]
    unversioned = [p for st, p in keep if st == "?"]
    missing     = [p for st, p in keep if st == "!"]
    if unversioned: run(["svn", "add"] + unversioned, cwd, dry)
    if missing:     run(["svn", "delete"] + missing,  cwd, dry)
    files = [p for _, p in keep]
    if not files: die("nothing to commit after filtering conflicts", 1)
    run(["svn", "commit"] + files + ["-m", opts["msg"]], cwd, dry)
    if not dry: print(query(["svn", "status"], cwd))

def main():
    opts = parse_args(sys.argv)
    cwd = Path(opts["dir"]).resolve()
    if not cwd.is_dir(): die(f"not a directory: {cwd}", 2)
    vcs = detect_vcs(cwd, opts["svn"])
    changes = git_changes(cwd) if vcs == "git" else svn_changes(cwd)
    if not changes: die("nothing to commit", 1)
    keep, dropped = apply_filter(changes, opts["inc"], opts["exc"])
    if dropped: print(f"Filtered ({len(dropped)}): {[p for _, p in dropped]}")
    if not keep: die("all changes filtered — nothing to commit", 1)
    print(f"Committing ({len(keep)}): {[p for _, p in keep]}")
    if vcs == "git": do_git(opts, cwd, keep)
    else:            do_svn(opts, cwd, keep)

if __name__ == "__main__":
    main()
