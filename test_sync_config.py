"""Tests for sync_config refactoring: src_key routing, subdirectory targets, opencode scanning."""

import sys
from pathlib import Path

import pytest

# Add tools/ to sys.path so we can import sync_config from its new location
sys.path.insert(0, str(Path(__file__).resolve().parent / "tools"))

from sync_config import (
    _build_table_row,
    _iter_items,
    copy_items,
    get_target,
    scan_sources,
)


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture()
def fake_home(tmp_path: Path) -> Path:
    """Create a fake home directory with claude + opencode + agents sources."""
    home = tmp_path / "home"

    # --- claude source ---
    claude = home / ".claude"
    (claude / "agents").mkdir(parents=True)
    (claude / "commands").mkdir(parents=True)
    (claude / "agents" / "coder.md").write_text("# coder agent", encoding="utf-8")
    (claude / "agents" / "reviewer.md").write_text("# reviewer agent", encoding="utf-8")
    (claude / "commands" / "commit.md").write_text("# commit command", encoding="utf-8")
    (claude / "commands" / "tools").mkdir()
    (claude / "commands" / "tools" / "helper.py").write_text("pass", encoding="utf-8")
    (claude / "CLAUDE.md").write_text("# Claude instructions", encoding="utf-8")

    # --- opencode source ---
    opencode = home / ".config" / "opencode"
    opencode.mkdir(parents=True)
    (opencode / "AGENTS.md").write_text("# Agents instructions", encoding="utf-8")
    (opencode / "agents").mkdir()
    (opencode / "agents" / "opencode-coder.md").write_text(
        "# opencode coder", encoding="utf-8"
    )
    (opencode / "commands").mkdir()
    (opencode / "commands" / "opencode-flow.md").write_text(
        "# opencode flow", encoding="utf-8"
    )

    # --- agents source ---
    agents_src = home / ".agents"
    (agents_src / "skills" / "myskill").mkdir(parents=True)
    (agents_src / "skills" / "myskill" / "SKILL.md").write_text(
        "# my skill", encoding="utf-8"
    )

    return home


@pytest.fixture()
def sources(fake_home: Path) -> dict[str, Path]:
    return {
        "claude": fake_home / ".claude",
        "agents": fake_home / ".agents",
        "opencode": fake_home / ".config" / "opencode",
    }


SKIP = {".gitignore", "__pycache__", "node_modules"}


# ---------------------------------------------------------------------------
# scan_sources
# ---------------------------------------------------------------------------


class TestScanSources:
    def test_claude_agents_and_commands_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        agents_paths = [p for p, _k in cats["agents"]]
        commands_paths = [p for p, _k in cats["commands"]]
        assert any(p.name == "coder.md" for p in agents_paths)
        assert any(p.name == "reviewer.md" for p in agents_paths)
        assert any(p.name == "commit.md" for p in commands_paths)

    def test_claude_config_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        config_names = [p.name for p, _k in cats["config"]]
        assert "CLAUDE.md" in config_names

    def test_opencode_agents_and_commands_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        agents_items = cats["agents"]
        commands_items = cats["commands"]
        assert any(
            p.name == "opencode-coder.md" and k == "opencode" for p, k in agents_items
        )
        assert any(
            p.name == "opencode-flow.md" and k == "opencode" for p, k in commands_items
        )

    def test_opencode_agents_md_in_config(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        config_items = cats["config"]
        assert any(p.name == "AGENTS.md" and k == "opencode" for p, k in config_items)

    def test_agents_skills_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        skill_names = [p.name for p, _k in cats["skills"]]
        assert "myskill" in skill_names

    def test_src_key_preserved_for_all_categories(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        for cat_key in ("agents", "commands", "skills", "config"):
            for _path, src_key in cats[cat_key]:
                assert src_key in sources

    def test_opencode_missing_agents_dir_still_finds_config(self, tmp_path: Path):
        """If opencode has no agents/ or commands/ dirs, AGENTS.md still found."""
        oc = tmp_path / "opencode"
        oc.mkdir()
        (oc / "AGENTS.md").write_text("# ag", encoding="utf-8")
        cats = scan_sources({"opencode": oc}, SKIP)
        assert len(cats["config"]) == 1
        assert cats["agents"] == []
        assert cats["commands"] == []


# ---------------------------------------------------------------------------
# get_target
# ---------------------------------------------------------------------------


class TestGetTarget:
    def test_agents_routed_to_src_key_subdirectory(self, tmp_path: Path):
        src = tmp_path / "source" / "coder.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "agents", "claude")
        assert result == Path.cwd() / "agents" / "claude" / "coder.md"

    def test_commands_routed_to_src_key_subdirectory(self, tmp_path: Path):
        src = tmp_path / "source" / "flow.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "commands", "opencode")
        assert result == Path.cwd() / "commands" / "opencode" / "flow.md"

    def test_skills_no_subdirectory(self, tmp_path: Path):
        src = tmp_path / "source" / "myskill"
        src.mkdir(parents=True)
        result = get_target(src, "skills", "agents")
        assert result == Path.cwd() / "skills" / "myskill"

    def test_config_goes_to_cwd_root(self, tmp_path: Path):
        src = tmp_path / "source" / "CLAUDE.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "config", "claude")
        assert result == Path.cwd() / "CLAUDE.md"


# ---------------------------------------------------------------------------
# _build_table_row — shows subdirectory-style paths
# ---------------------------------------------------------------------------


class TestBuildTableRow:
    def test_agents_row_shows_src_key_path(self, tmp_path: Path):
        src = tmp_path / "source" / "coder.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        row = _build_table_row(src, "agents", "agents/claude/coder.md")
        assert "agents/claude/coder.md" in row

    def test_commands_row_shows_src_key_path(self, tmp_path: Path):
        src = tmp_path / "source" / "flow.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        row = _build_table_row(src, "commands", "commands/opencode/flow.md")
        assert "commands/opencode/flow.md" in row


# ---------------------------------------------------------------------------
# _confirm / copy_items with triples (src_key)
# ---------------------------------------------------------------------------


class TestCopyItemsWithSrcKey:
    def test_copy_agent_to_subdirectory(self, tmp_path: Path, monkeypatch):
        """Verify copy_items routes agents into src_key subdirectory."""
        src = tmp_path / "src" / "coder.md"
        src.parent.mkdir(parents=True)
        src.write_text("# agent", encoding="utf-8")

        dest_cwd = tmp_path / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        # selected is now list of (Path, cat_key, src_key) triples
        selected = [(src, "agents", "claude")]
        copy_items(selected, dry_run=True)

        # In dry-run mode nothing is actually copied, but the printed
        # target path should include the subdirectory
        # Verify get_target returns the right path
        target = get_target(src, "agents", "claude")
        assert target == dest_cwd / "agents" / "claude" / "coder.md"

    def test_actual_copy_agent_to_subdirectory(self, tmp_path: Path, monkeypatch):
        """Verify actual copy works with subdirectory routing."""
        src = tmp_path / "src" / "reviewer.md"
        src.parent.mkdir(parents=True)
        src.write_text("# reviewer agent", encoding="utf-8")

        dest_cwd = tmp_path / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        selected = [(src, "agents", "opencode")]
        copy_items(selected, dry_run=False)

        expected = dest_cwd / "agents" / "opencode" / "reviewer.md"
        assert expected.exists()
        assert expected.read_text(encoding="utf-8") == "# reviewer agent"

    def test_copy_command_to_subdirectory(self, tmp_path: Path, monkeypatch):
        """Verify commands get routed to src_key subdirectory."""
        src = tmp_path / "src" / "flow.md"
        src.parent.mkdir(parents=True)
        src.write_text("# flow command", encoding="utf-8")

        dest_cwd = tmp_path / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        selected = [(src, "commands", "claude")]
        copy_items(selected, dry_run=False)

        expected = dest_cwd / "commands" / "claude" / "flow.md"
        assert expected.exists()

    def test_copy_skill_no_subdirectory(self, tmp_path: Path, monkeypatch):
        """Skills should NOT be routed into a src_key subdirectory."""
        src = tmp_path / "src" / "myskill"
        src.mkdir(parents=True)
        (src / "SKILL.md").write_text("# skill", encoding="utf-8")

        dest_cwd = tmp_path / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        selected = [(src, "skills", "agents")]
        copy_items(selected, dry_run=False)

        expected = dest_cwd / "skills" / "myskill" / "SKILL.md"
        assert expected.exists()

    def test_copy_config_to_root(self, tmp_path: Path, monkeypatch):
        """Config files go to CWD root."""
        src = tmp_path / "src" / "CLAUDE.md"
        src.parent.mkdir(parents=True)
        src.write_text("# claude", encoding="utf-8")

        dest_cwd = tmp_path / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        selected = [(src, "config", "claude")]
        copy_items(selected, dry_run=False)

        expected = dest_cwd / "CLAUDE.md"
        assert expected.exists()


# ---------------------------------------------------------------------------
# End-to-end: scan → select (mock) → copy
# ---------------------------------------------------------------------------


class TestEndToEnd:
    def test_full_flow_claude_and_opencode(
        self, fake_home: Path, sources: dict[str, Path], monkeypatch
    ):
        """Simulate full sync flow: scan, then copy selected items."""
        dest_cwd = fake_home / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        cats = scan_sources(sources, SKIP)

        # Pick a claude agent + opencode agent + opencode command
        selected = []
        for p, k in cats["agents"]:
            if k == "claude" and p.name == "coder.md":
                selected.append((p, "agents", k))
            if k == "opencode" and p.name == "opencode-coder.md":
                selected.append((p, "agents", k))
        for p, k in cats["commands"]:
            if k == "opencode" and p.name == "opencode-flow.md":
                selected.append((p, "commands", k))

        assert len(selected) == 3
        copy_items(selected, dry_run=False)

        # Verify subdirectory routing
        assert (dest_cwd / "agents" / "claude" / "coder.md").exists()
        assert (dest_cwd / "agents" / "opencode" / "opencode-coder.md").exists()
        assert (dest_cwd / "commands" / "opencode" / "opencode-flow.md").exists()
