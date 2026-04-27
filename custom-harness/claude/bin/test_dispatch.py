"""
Tests for dispatch.py Phase 1 changes.

TDD red phase: tests describe desired behavior before implementation.
Run with: pytest test_dispatch.py -v
"""
import importlib
import json
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

DISPATCH_PY = Path(__file__).parent / "dispatch.py"
PYTHON = sys.executable


def run_dispatch(*args) -> tuple[int, str, str]:
    r = subprocess.run(
        [PYTHON, str(DISPATCH_PY), *args],
        capture_output=True, text=True, encoding="utf-8",
    )
    return r.returncode, r.stdout, r.stderr


# ---------------------------------------------------------------------------
# Test 1 — thin pointer mode (default)
# ---------------------------------------------------------------------------

def test_thin_pointer_default():
    """Default output must point to file, not embed body."""
    rc, stdout, stderr = run_dispatch("vsc-committer", "test prompt")
    assert rc == 0, f"dispatch failed: {stderr}"
    data = json.loads(stdout)
    prompt = data["payloads"][0]["prompt"]
    assert "DEFINITION_FILE:" in prompt, "Thin pointer must include DEFINITION_FILE:"
    assert "<!-- BEGIN:" not in prompt, "Inline body marker must not appear in default mode"


# ---------------------------------------------------------------------------
# Test 2 — --inline preserves old behaviour
# ---------------------------------------------------------------------------

def test_inline_mode():
    """--inline must embed body with <!-- BEGIN: marker."""
    rc, stdout, stderr = run_dispatch("--inline", "vsc-committer", "test prompt")
    assert rc == 0, f"dispatch --inline failed: {stderr}"
    data = json.loads(stdout)
    prompt = data["payloads"][0]["prompt"]
    assert "<!-- BEGIN:" in prompt, "--inline must include <!-- BEGIN: body marker"
    assert "DEFINITION_FILE:" not in prompt, "Thin pointer must not appear in --inline mode"


# ---------------------------------------------------------------------------
# Test 3 — parallel MD dedup (unit test via import)
# ---------------------------------------------------------------------------

def test_parallel_md_dedup():
    """build_payload with shared _md_cache must call parse_md once per unique path."""
    # Import fresh module so HARNESS_DIR resolves correctly
    sys.path.insert(0, str(DISPATCH_PY.parent))
    import dispatch
    importlib.reload(dispatch)

    registry = dispatch.load_registry()
    md_cache: dict = {}
    call_count = {"n": 0}
    orig_parse_md = dispatch.parse_md

    def counting_parse_md(path):
        call_count["n"] += 1
        return orig_parse_md(path)

    with patch.object(dispatch, "parse_md", side_effect=counting_parse_md):
        dispatch.build_payload(registry, "vsc-committer", "prompt1", _md_cache=md_cache)
        dispatch.build_payload(registry, "vsc-committer", "prompt2", _md_cache=md_cache)

    assert call_count["n"] == 1, (
        f"parse_md called {call_count['n']} time(s); expected 1 for same path"
    )


# ---------------------------------------------------------------------------
# Test 4 — --help shows [commands] section
# ---------------------------------------------------------------------------

def test_help_shows_commands():
    """--help output must list [commands] section."""
    rc, stdout, stderr = run_dispatch("--help")
    assert rc == 0, f"dispatch --help failed: {stderr}"
    assert "[commands]" in stdout, "--help must display [commands] section"


# ---------------------------------------------------------------------------
# Test 5 — trailing colon on name falls through to fuzzy match
# ---------------------------------------------------------------------------

def test_trailing_colon_resolved():
    """'dev-cycle:' (trailing colon, no name) must fall through to fuzzy match."""
    rc, stdout, stderr = run_dispatch("dev-cycle:", "test prompt")
    assert rc == 0, f"dev-cycle: dispatch failed: {stderr}"
    data = json.loads(stdout)
    assert data["payloads"][0]["description"], "Resolved agent must have non-empty description"
