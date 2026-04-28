"""Tests for vsc-commit.py boundary-scoping fixes."""
import importlib.util, shutil, tempfile
from pathlib import Path
from unittest.mock import patch

# vsc-commit.py has a hyphen — load via spec
_script = Path(__file__).parent / "vsc-commit.py"
_spec = importlib.util.spec_from_file_location("vsc_commit", _script)
mod = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(mod)


# ── Fix 1: git status must include '-- .' pathspec ────────────────────────────

def test_git_changes_uses_pathspec_dot():
    """git_changes must pass '-- .' so only DIR-subtree changes are returned."""
    captured = []

    def fake_query(cmd, cwd):
        captured.append(list(cmd))
        return ""

    with patch.object(mod, "query", fake_query):
        mod.git_changes(Path("."))

    assert len(captured) == 1
    cmd = captured[0]
    assert "--" in cmd, "git status must include '--' separator"
    sep = cmd.index("--")
    assert "." in cmd[sep + 1:], "git status must include '.' pathspec after '--'"


# ── Fix 2: git commit must include '-- <files>' pathspec ─────────────────────

def test_git_commit_uses_file_pathspec():
    """do_git must pass '-- <files>' to commit so pre-staged outside files are excluded."""
    run_calls = []

    def fake_run(cmd, cwd, dry=False):
        run_calls.append(list(cmd))
        return ""

    opts = {"msg": "✨ feat(x): test", "push": False, "dry": False}
    keep = [("M", "src/main.py"), ("A", "src/new.py")]

    with patch.object(mod, "run", fake_run):
        mod.do_git(opts, Path("."), keep)

    commit_cmd = next((c for c in run_calls if "commit" in c), None)
    assert commit_cmd is not None, "git commit was not called"
    assert "--" in commit_cmd, "git commit must include '--' pathspec separator"
    sep = commit_cmd.index("--")
    files_in_cmd = commit_cmd[sep + 1:]
    assert "src/main.py" in files_in_cmd
    assert "src/new.py" in files_in_cmd


# ── Fix 3: apply_filter must drop paths outside cwd ──────────────────────────

def test_apply_filter_drops_paths_outside_cwd():
    """apply_filter must drop paths that resolve outside the target cwd."""
    tmp = Path(tempfile.mkdtemp())
    try:
        sub_a = tmp / "sub_a"
        sub_a.mkdir()
        changes = [
            ("M", "../sub_b/file.py"),   # outside cwd
            ("M", "local.py"),           # inside cwd
            ("A", "deep/nested.py"),     # inside cwd
        ]
        keep, dropped = mod.apply_filter(changes, [], [], sub_a)
        paths_kept    = [p for _, p in keep]
        paths_dropped = [p for _, p in dropped]
        assert "local.py" in paths_kept,          "local.py should be kept"
        assert "deep/nested.py" in paths_kept,    "deep/nested.py should be kept"
        assert "../sub_b/file.py" in paths_dropped, "../sub_b/file.py must be dropped"
    finally:
        shutil.rmtree(tmp)
