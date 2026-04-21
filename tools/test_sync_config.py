"""Tests for sync_config: custom-harness scanning, skills routing, config discovery."""

from pathlib import Path

import pytest

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
    home = tmp_path / "home"

    # --- claude source ---
    claude = home / ".claude"
    (claude / "custom-harness" / "agents").mkdir(parents=True)
    (claude / "custom-harness" / "commands").mkdir(parents=True)
    (claude / "custom-harness" / "agents" / "coder.md").write_text(
        "# coder agent", encoding="utf-8"
    )
    (claude / "custom-harness" / "agents" / "reviewer.md").write_text(
        "# reviewer agent", encoding="utf-8"
    )
    (claude / "custom-harness" / "commands" / "commit.md").write_text(
        "# commit command", encoding="utf-8"
    )
    (claude / "CLAUDE.md").write_text("# Claude instructions", encoding="utf-8")

    # --- opencode source ---
    opencode = home / ".config" / "opencode"
    (opencode / "custom-harness" / "agents").mkdir(parents=True)
    (opencode / "custom-harness" / "commands").mkdir(parents=True)
    (opencode / "custom-harness" / "agents" / "opencode-coder.md").write_text(
        "# opencode coder", encoding="utf-8"
    )
    (opencode / "custom-harness" / "commands" / "opencode-flow.md").write_text(
        "# opencode flow", encoding="utf-8"
    )
    (opencode / "AGENTS.md").write_text("# Agents instructions", encoding="utf-8")

    # --- agents source (needs custom-harness/ to avoid early return in _scan_harness) ---
    agents_src = home / ".agents"
    (agents_src / "custom-harness").mkdir(parents=True)
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
# _iter_items
# ---------------------------------------------------------------------------


class TestIterItems:
    def test_yields_non_skipped_files(self, tmp_path: Path):
        (tmp_path / "a.md").write_text("a", encoding="utf-8")
        (tmp_path / "b.py").write_text("b", encoding="utf-8")
        names = {p.name for p in _iter_items(tmp_path, SKIP)}
        assert names == {"a.md", "b.py"}

    def test_skips_hidden_and_skip_set(self, tmp_path: Path):
        (tmp_path / ".hidden").write_text("h", encoding="utf-8")
        (tmp_path / "__pycache__").mkdir()
        (tmp_path / "good.md").write_text("g", encoding="utf-8")
        names = {p.name for p in _iter_items(tmp_path, SKIP)}
        assert names == {"good.md"}

    def test_nonexistent_dir_yields_nothing(self, tmp_path: Path):
        assert list(_iter_items(tmp_path / "nope", SKIP)) == []


# ---------------------------------------------------------------------------
# scan_sources
# ---------------------------------------------------------------------------


class TestScanSources:
    def test_claude_harness_agents_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        agents_names = [p.name for p, _sk, _cat, _fh in cats["agents"]]
        assert "coder.md" in agents_names
        assert "reviewer.md" in agents_names

    def test_claude_harness_commands_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        commands_names = [p.name for p, _sk, _cat, _fh in cats["commands"]]
        assert "commit.md" in commands_names

    def test_claude_config_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        config_names = [p.name for p, _sk, _cat, _fh in cats["config"]]
        assert "CLAUDE.md" in config_names

    def test_opencode_harness_agents_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        assert any(
            p.name == "opencode-coder.md" and sk == "opencode"
            for p, sk, _cat, _fh in cats["agents"]
        )

    def test_opencode_harness_commands_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        assert any(
            p.name == "opencode-flow.md" and sk == "opencode"
            for p, sk, _cat, _fh in cats["commands"]
        )

    def test_opencode_agents_md_in_config(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        assert any(
            p.name == "AGENTS.md" and sk == "opencode"
            for p, sk, _cat, _fh in cats["config"]
        )

    def test_agents_skills_found(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        skill_names = [p.name for p, _sk, _cat, _fh in cats["skills"]]
        assert "myskill" in skill_names

    def test_harness_items_marked_from_harness(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        for cat_key in ("agents", "commands"):
            for p, sk, cat, fh in cats[cat_key]:
                assert fh is True
                assert cat == cat_key

    def test_skills_items_not_from_harness(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        for p, sk, cat, fh in cats["skills"]:
            assert fh is False
            assert cat == "skills"

    def test_config_items_not_from_harness(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        for p, sk, cat, fh in cats["config"]:
            assert fh is False

    def test_src_key_preserved(self, sources: dict[str, Path]):
        cats = scan_sources(sources, SKIP)
        for cat_key in ("agents", "commands", "skills", "config"):
            for p, sk, cat, fh in cats[cat_key]:
                assert sk in sources

    def test_opencode_no_harness_still_finds_config(self, tmp_path: Path):
        oc = tmp_path / "opencode"
        oc.mkdir()
        (oc / "AGENTS.md").write_text("# ag", encoding="utf-8")
        cats = scan_sources({"opencode": oc}, SKIP)
        assert len(cats["config"]) == 1
        assert cats.get("agents", []) == []
        assert cats.get("commands", []) == []


# ---------------------------------------------------------------------------
# get_target
# ---------------------------------------------------------------------------


class TestGetTarget:
    def test_harness_agents_routed_to_custom_harness(self, tmp_path: Path):
        src = tmp_path / "source" / "coder.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "agents", "claude", from_harness=True)
        assert result == Path.cwd() / "custom-harness" / "claude" / "agents" / "coder.md"

    def test_harness_commands_routed_to_custom_harness(self, tmp_path: Path):
        src = tmp_path / "source" / "flow.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "commands", "opencode", from_harness=True)
        assert result == Path.cwd() / "custom-harness" / "opencode" / "commands" / "flow.md"

    def test_harness_file_goes_to_harness_dir(self, tmp_path: Path):
        src = tmp_path / "source" / "setup.sh"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "harness", "claude", from_harness=True)
        assert result == Path.cwd() / "custom-harness" / "claude" / "setup.sh"

    def test_skills_routed_with_src_key(self, tmp_path: Path):
        src = tmp_path / "source" / "myskill"
        src.mkdir(parents=True)
        result = get_target(src, "skills", "agents", from_harness=False)
        assert result == Path.cwd() / "skills" / "agents" / "myskill"

    def test_config_goes_to_cwd_root(self, tmp_path: Path):
        src = tmp_path / "source" / "CLAUDE.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "config", "claude", from_harness=False)
        assert result == Path.cwd() / "CLAUDE.md"

    def test_non_harness_non_skill_goes_to_root(self, tmp_path: Path):
        src = tmp_path / "source" / "readme.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        result = get_target(src, "config", "claude", from_harness=False)
        assert result == Path.cwd() / "readme.md"


# ---------------------------------------------------------------------------
# _build_table_row
# ---------------------------------------------------------------------------


class TestBuildTableRow:
    def test_agents_row(self, tmp_path: Path):
        src = tmp_path / "source" / "coder.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        row = _build_table_row(src, "agents", "custom-harness/claude/agents/coder.md")
        assert "custom-harness/claude/agents/coder.md" in row

    def test_commands_row(self, tmp_path: Path):
        src = tmp_path / "source" / "flow.md"
        src.parent.mkdir(parents=True)
        src.write_text("x", encoding="utf-8")
        row = _build_table_row(src, "commands", "custom-harness/opencode/commands/flow.md")
        assert "custom-harness/opencode/commands/flow.md" in row


# ---------------------------------------------------------------------------
# copy_items with 4-tuples
# ---------------------------------------------------------------------------


class TestCopyItems:
    def test_dry_run_harness_agent(self, tmp_path: Path, monkeypatch):
        src = tmp_path / "src" / "coder.md"
        src.parent.mkdir(parents=True)
        src.write_text("# agent", encoding="utf-8")
        dest = tmp_path / "project"
        dest.mkdir()
        monkeypatch.chdir(dest)
        selected = [(src, "agents", "claude", True)]
        copy_items(selected, dry_run=True)
        assert not (dest / "custom-harness" / "claude" / "agents" / "coder.md").exists()

    def test_actual_copy_harness_agent(self, tmp_path: Path, monkeypatch):
        src = tmp_path / "src" / "reviewer.md"
        src.parent.mkdir(parents=True)
        src.write_text("# reviewer agent", encoding="utf-8")
        dest = tmp_path / "project"
        dest.mkdir()
        monkeypatch.chdir(dest)
        selected = [(src, "agents", "opencode", True)]
        copy_items(selected, dry_run=False)
        expected = dest / "custom-harness" / "opencode" / "agents" / "reviewer.md"
        assert expected.exists()
        assert expected.read_text(encoding="utf-8") == "# reviewer agent"

    def test_copy_harness_command(self, tmp_path: Path, monkeypatch):
        src = tmp_path / "src" / "flow.md"
        src.parent.mkdir(parents=True)
        src.write_text("# flow command", encoding="utf-8")
        dest = tmp_path / "project"
        dest.mkdir()
        monkeypatch.chdir(dest)
        selected = [(src, "commands", "claude", True)]
        copy_items(selected, dry_run=False)
        assert (dest / "custom-harness" / "claude" / "commands" / "flow.md").exists()

    def test_copy_skill_directory(self, tmp_path: Path, monkeypatch):
        src = tmp_path / "src" / "myskill"
        src.mkdir(parents=True)
        (src / "SKILL.md").write_text("# skill", encoding="utf-8")
        dest = tmp_path / "project"
        dest.mkdir()
        monkeypatch.chdir(dest)
        selected = [(src, "skills", "agents", False)]
        copy_items(selected, dry_run=False)
        assert (dest / "skills" / "agents" / "myskill" / "SKILL.md").exists()

    def test_copy_config_to_root(self, tmp_path: Path, monkeypatch):
        src = tmp_path / "src" / "CLAUDE.md"
        src.parent.mkdir(parents=True)
        src.write_text("# claude", encoding="utf-8")
        dest = tmp_path / "project"
        dest.mkdir()
        monkeypatch.chdir(dest)
        selected = [(src, "config", "claude", False)]
        copy_items(selected, dry_run=False)
        assert (dest / "CLAUDE.md").exists()


# ---------------------------------------------------------------------------
# End-to-end: scan → copy
# ---------------------------------------------------------------------------


class TestEndToEnd:
    def test_full_flow(
        self, fake_home: Path, sources: dict[str, Path], monkeypatch
    ):
        dest_cwd = fake_home / "project"
        dest_cwd.mkdir()
        monkeypatch.chdir(dest_cwd)

        cats = scan_sources(sources, SKIP)

        selected = []
        for p, sk, cat, fh in cats["agents"]:
            if p.name == "coder.md":
                selected.append((p, cat, sk, fh))
        for p, sk, cat, fh in cats["commands"]:
            if p.name == "opencode-flow.md":
                selected.append((p, cat, sk, fh))
        for p, sk, cat, fh in cats["skills"]:
            selected.append((p, cat, sk, fh))

        assert len(selected) >= 3
        copy_items(selected, dry_run=False)

        assert (dest_cwd / "custom-harness" / "claude" / "agents" / "coder.md").exists()
        assert (dest_cwd / "custom-harness" / "opencode" / "commands" / "opencode-flow.md").exists()
        assert (dest_cwd / "skills" / "agents" / "myskill" / "SKILL.md").exists()
