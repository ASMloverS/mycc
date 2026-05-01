#!/usr/bin/env python3
"""sync_config.py - 交互式拷贝用户域 AI 编码工具配置到当前目录"""

import argparse
import datetime
import os
import re
import shutil
import sys
import types
import unicodedata
from collections.abc import Callable, Iterator
from pathlib import Path

try:
    import yaml
except ImportError:
    yaml: types.ModuleType | None = None

HOME = Path.home()

DEFAULT_SOURCES = {
    "claude": HOME / ".claude",
    "opencode": HOME / ".config" / "opencode",
}
DEFAULT_SRC_LABELS = {
    "claude": "~/.claude",
    "opencode": "~/.config/opencode",
}
DEFAULT_SKIP = {
    ".credentials.json",
    "mcp-needs-auth-cache.json",
    "stats-cache.json",
    "history.jsonl",
    ".skill-lock.json",
    "bun.lock",
    "package-lock.json",
    ".gitignore",
    "debug",
    "cache",
    "backups",
    "file-history",
    "paste-cache",
    "plans",
    "plugins",
    "projects",
    "session-env",
    "sessions",
    "shell-snapshots",
    "statusline-pro",
    "tasks",
    "telemetry",
    "todos",
    "usage-data",
    "output-styles",
    "node_modules",
    "__pycache__",
}

CONFIG_FILE_MAP = {
    "claude": "CLAUDE.md",
    "opencode": "AGENTS.md",
}

MODEL_OPTIONS = [
    "(keep original)",
    "claude-sonnet-4-6",
    "claude-opus-4",
    "claude-haiku-3",
    "claude-sonnet-4",
    "zai-coding-plan/glm-5.1",
    "deepseek-chat",
    "gpt-4.1",
    "o4-mini",
    "(manual input...)",
]

CAT_LABELS = {
    "agents": "Agents",
    "commands": "Commands",
    "skills": "Skills",
    "bin": "Bin",
    "harness": "Harness",
    "config": "配置文件",
}
CAT_COLORS = {
    "agents": "\033[36m",
    "commands": "\033[33m",
    "skills": "\033[35m",
    "bin": "\033[34m",
    "harness": "\033[36m",
    "config": "\033[32m",
}
RST = "\033[0m"


def _enable_vt100() -> None:
    if sys.platform != "win32":
        return
    import ctypes

    k = ctypes.windll.kernel32
    h = k.GetStdHandle(-11)
    m = ctypes.c_ulong()
    k.GetConsoleMode(h, ctypes.byref(m))
    k.SetConsoleMode(h, m.value | 0x0004)


def load_config() -> tuple[dict[str, Path], dict[str, str], set[str]]:
    script_dir = Path(__file__).resolve().parent
    config_path = script_dir / "sync_config.yaml"
    sources = dict(DEFAULT_SOURCES)
    src_labels = dict(DEFAULT_SRC_LABELS)
    skip = set(DEFAULT_SKIP)
    if not config_path.exists():
        return sources, src_labels, skip
    if yaml is None:
        print("提示: 发现 sync_config.yaml 但未安装 PyYAML，使用默认配置")
        print("  安装: pip install pyyaml\n")
        return sources, src_labels, skip
    with open(config_path, "r", encoding="utf-8") as f:
        data = yaml.safe_load(f) or {}
    if "sources" in data:
        sources = {}
        src_labels = {}
        for key, raw in data["sources"].items():
            p = Path(os.path.expanduser(str(raw)))
            sources[key] = p
            src_labels[key] = str(raw)
    if "skip" in data:
        skip = set(str(s) for s in data["skip"])
    return sources, src_labels, skip


# --- Keyboard Input ---

_KEY_MAP_COMMON = {
    "\r": "enter",
    "\n": "enter",
    " ": "space",
    "q": "q",
    "Q": "q",
    "y": "y",
    "Y": "y",
    "n": "n",
    "N": "n",
}
_KEY_MAP_WIN_EXT = {"H": "up", "P": "down", "K": "left", "M": "right"}
_KEY_MAP_LINUX_CSI = {"A": "up", "B": "down", "D": "left", "C": "right"}


def _get_key_windows() -> str:
    import msvcrt

    ch = msvcrt.getwch()
    if ch in ("\x00", "\xe0"):
        ch2 = msvcrt.getwch()
        return _KEY_MAP_WIN_EXT.get(ch2, f"special:{ch2}")
    return _KEY_MAP_COMMON.get(ch, ch)


def _get_key_linux() -> str:
    import termios
    import tty

    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
        if ch == "\x1b":
            ch2 = sys.stdin.read(1)
            if ch2 == "[":
                ch3 = sys.stdin.read(1)
                return _KEY_MAP_LINUX_CSI.get(ch3, f"esc[{ch3}")
            return f"esc:{ch2}"
        return _KEY_MAP_COMMON.get(ch, ch)
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)


get_key: Callable[[], str] = (
    _get_key_windows if sys.platform == "win32" else _get_key_linux
)


# --- Emoji Checkbox ---


def _term_height() -> int:
    try:
        return os.get_terminal_size().lines
    except OSError:
        return 24


_ANSI_RE = re.compile(r"\033\[[0-9;]*m")


def _interactive_list(
    items: list[str],
    message: str,
    mode: str = "checkbox",
    default: int = 0,
) -> list[str] | str | None:
    n = len(items)
    cursor = 0 if mode == "checkbox" else max(0, min(default, n - 1))
    selected = [False] * n if mode == "checkbox" else None
    visible_rows = max(_term_height() - 4, 5)
    scroll_top = 0
    prev_lines = 0
    while True:
        if cursor < scroll_top:
            scroll_top = cursor
        elif cursor >= scroll_top + visible_rows:
            scroll_top = cursor - visible_rows + 1
        display_count = min(visible_rows, n - scroll_top)
        if prev_lines > 0:
            sys.stdout.write(f"\033[{prev_lines}A")
        sys.stdout.write("\033[J")
        lines = 0
        indicator = f" ({cursor + 1}/{n})" if n > visible_rows else ""
        sys.stdout.write(f"\033[1m{message}{indicator}\033[0m\n")
        lines += 1
        for vi in range(display_count):
            i = scroll_top + vi
            if mode == "checkbox":
                mark = "✅" if selected[i] else "⬜"
            else:
                mark = "●" if i == cursor else "○"
            pointer = "▶" if i == cursor else " "
            sys.stdout.write(f" {pointer} {mark} {items[i]}\n")
            lines += 1
        if n > visible_rows:
            bar_pos = int(visible_rows * cursor / max(n - 1, 1))
            bar_chars = ["─"] * visible_rows
            bar_chars[bar_pos] = "█"
            sys.stdout.write(f"  {''.join(bar_chars)}\n")
            lines += 1
        sys.stdout.flush()
        prev_lines = lines
        key = get_key()
        if key == "up":
            cursor = (cursor - 1) % n
        elif key == "down":
            cursor = (cursor + 1) % n
        elif key == "space":
            if mode == "checkbox":
                selected[cursor] = not selected[cursor]
            else:
                sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
                return items[cursor]
        elif key == "enter":
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            if mode == "checkbox":
                return [items[i] for i in range(n) if selected[i]]
            return items[cursor]
        elif key == "q":
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            return [] if mode == "checkbox" else None


def emoji_checkbox(
    items: list[str],
    message: str = "选择要拷贝的项目 (↑↓移动, 空格切换, 回车确认, q退出)",
) -> list[str]:
    return _interactive_list(items, message, mode="checkbox")  # type: ignore[return-value]


def emoji_radiolist(
    items: list[str],
    message: str = "选择一个选项 (↑↓移动, 空格/回车确认, q退出)",
    default: int = 0,
) -> str | None:
    return _interactive_list(items, message, mode="radio", default=default)  # type: ignore[return-value]


# --- Custom Confirm ---


def custom_confirm(message: str = "确认执行？", default: bool = True) -> bool:
    opts = ["✅ 是", "❌ 否"]
    cursor = 0 if default else 1
    prev_lines = 0
    while True:
        if prev_lines > 0:
            sys.stdout.write(f"\033[{prev_lines}A")
        sys.stdout.write("\033[J")
        lines = 0
        sys.stdout.write(f"\033[1m{message}\033[0m\n")
        lines += 1
        for i, opt in enumerate(opts):
            pointer = "▶" if i == cursor else " "
            sys.stdout.write(f" {pointer} {opt}\n")
            lines += 1
        sys.stdout.flush()
        prev_lines = lines
        key = get_key()
        if key in ("left", "right", "up", "down"):
            cursor = 1 - cursor
        elif key == "enter":
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            return cursor == 0
        elif key == "y":
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            return True
        elif key in ("n", "q"):
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            return False


# --- Scanning ---


def _iter_items(directory: Path, skip: set[str]) -> Iterator[Path]:
    if not directory.exists():
        return
    for p in directory.iterdir():
        if p.name in skip or p.name.startswith("."):
            continue
        yield p


def _scan_harness(
    src_root: Path, src_key: str, skip: set[str], cats: dict
) -> None:
    ch = src_root / "custom-harness"
    if not ch.exists():
        return
    for p in _iter_items(ch, skip):
        if p.is_dir():
            for item in _iter_items(p, skip):
                cat = p.name
                cats.setdefault(cat, []).append((item, src_key, cat, True))
        else:
            cats.setdefault("harness", []).append((p, src_key, "harness", True))
    sk = src_root / "skills"
    if sk.exists():
        for p in _iter_items(sk, skip):
            cats.setdefault("skills", []).append((p, src_key, "skills", False))


def scan_sources(
    sources: dict[str, Path], skip: set[str]
) -> dict[str, list[tuple[Path, str, str, bool]]]:
    cats: dict[str, list[tuple[Path, str, str, bool]]] = {}
    for src_key, src_root in sources.items():
        if not src_root.exists():
            continue
        _scan_harness(src_root, src_key, skip, cats)
        if src_key in CONFIG_FILE_MAP:
            fp = src_root / CONFIG_FILE_MAP[src_key]
            if fp.exists():
                cats.setdefault("config", []).append((fp, src_key, "", False))
    return cats


def _fmt_size(n: float) -> str:
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024:
            return f"{n:.0f}{unit}" if unit == "B" else f"{n:.1f}{unit}"
        n /= 1024
    return f"{n:.1f}TB"


def _item_detail(
    path: Path, src_key: str, sources: dict[str, Path], src_labels: dict[str, str]
) -> tuple[Path, str, str, str]:
    kind = "目录" if path.is_dir() else "文件"
    try:
        if path.is_dir():
            total = sum(
                f.stat().st_size for f in path.rglob("*") if f.is_file()
            )
        else:
            total = path.stat().st_size
        mtime = datetime.datetime.fromtimestamp(path.stat().st_mtime).strftime(
            "%Y-%m-%d %H:%M"
        )
    except (FileNotFoundError, OSError):
        total = 0
        mtime = "未知"
    size = _fmt_size(total)
    rel = path.relative_to(sources[src_key])
    return rel, kind, size, mtime


def interactive_select(
    categories: dict[str, list[tuple[Path, str, str, bool]]],
    sources: dict[str, Path],
    src_labels: dict[str, str],
) -> list[tuple[Path, str, str, bool]]:
    all_labels: list[str] = []
    label_map: dict[str, tuple[Path, str, str, bool]] = {}
    _DISPLAY_ORDER = ("harness", "bin", "agents", "commands", "skills", "config")
    cat_keys = [k for k in _DISPLAY_ORDER if k in categories]
    cat_keys += [k for k in categories if k not in _DISPLAY_ORDER]
    for cat_key in cat_keys:
        items = categories.get(cat_key, [])
        if not items:
            continue
        c = CAT_COLORS.get(cat_key, "")
        cat_label = CAT_LABELS.get(cat_key, cat_key.title())
        for path, src_key, _sub, from_harness in items:
            rel, kind, size, mtime = _item_detail(
                path, src_key, sources, src_labels
            )
            tag = f"{c}[{cat_label}]{RST}"
            harness_tag = " [harness]" if from_harness else ""
            label = (
                f"{tag}{harness_tag} {rel}  [{kind}]  {size}"
                f"  {mtime}  ({src_labels[src_key]})"
            )
            all_labels.append(label)
            label_map[label] = (path, cat_key, src_key, from_harness)
    if not all_labels:
        return []
    chosen = emoji_checkbox(all_labels)
    return [label_map[c] for c in chosen]


def _strwidth(s: str) -> int:
    return sum(2 if unicodedata.east_asian_width(ch) in ("W", "F") else 1 for ch in s)


TABLE_HEADERS = ["类别", "名称", "类型", "目标路径"]


def _rel_path(path: Path, base: Path) -> str:
    try:
        return str(path.relative_to(base))
    except ValueError:
        return str(path)


def _remove_path(path: Path) -> None:
    if not path.exists():
        return
    if path.is_dir():
        shutil.rmtree(path)
    else:
        path.unlink()


def _check_overwrite(dst: Path, new_content: str) -> bool:
    if not dst.exists():
        return True
    existing = dst.read_text(encoding="utf-8")
    if existing == new_content:
        print(f"  跳过 (内容相同): {dst}")
        return False
    if not custom_confirm(f"目标已存在: {dst}\n是否覆盖？", default=False):
        print(f"  跳过: {dst}")
        return False
    return True


def _build_table_row(src: Path, cat: str, rel_dst: str) -> list[str]:
    c = CAT_COLORS.get(cat, "")
    return [
        f"{c}{CAT_LABELS.get(cat, cat.title())}{RST}",
        str(src.name),
        "目录" if src.is_dir() else "文件",
        rel_dst,
    ]


def _strip_ansi(s: str) -> str:
    return _ANSI_RE.sub("", s)


def _print_table(rows: list[list[str]], headers: list[str]) -> None:
    widths = [_strwidth(h) for h in headers]
    stripped = []
    for row in rows:
        srow = []
        for j, cell in enumerate(row):
            sc = _strip_ansi(cell)
            srow.append(sc)
            widths[j] = max(widths[j], _strwidth(sc))
        stripped.append(srow)
    hline = lambda l, r, j: l + j.join("─" * (w + 2) for w in widths) + r
    top, mid, bot = hline("┌", "┬", "┐"), hline("├", "┼", "┤"), hline("└", "┴", "┘")

    def fmt(cells, colored=None):
        parts = []
        for j, sc in enumerate(cells):
            pad = " " * (widths[j] - _strwidth(sc))
            if colored and j < len(colored):
                parts.append(f" {colored[j]}{pad}{RST} ")
            else:
                parts.append(f" {sc}{pad} ")
        return "│" + "│".join(parts) + "│"

    print(f"\n  {top}")
    print(f"  {fmt(headers)}")
    print(f"  {mid}")
    for i, srow in enumerate(stripped):
        print(f"  {fmt(srow, rows[i])}")
    print(f"  {bot}")


# --- Frontmatter ---


def parse_frontmatter(content: str) -> tuple[dict, str]:
    """解析 Markdown 文件的 YAML frontmatter，返回 (元数据字典, 正文)"""
    if not content.startswith("---"):
        return {}, content
    m = re.match(r"---[ \t]*\n(.*?)\n---[ \t]*\n", content, re.DOTALL)
    if m is None:
        return {}, content
    fm_text = m.group(1)
    body = content[m.end():]
    if yaml is None:
        return {}, content
    try:
        meta = yaml.safe_load(fm_text) or {}
    except yaml.YAMLError:
        meta = {}
    if not isinstance(meta, dict):
        meta = {}
    return meta, body


def write_frontmatter(content: str, overrides: dict) -> str:
    """修改 Markdown 内容的 frontmatter 字段，返回修改后的完整内容"""
    meta, body = parse_frontmatter(content)
    meta.update(overrides)
    if not meta:
        return body
    fm_text = yaml.dump(
        meta, allow_unicode=True, default_flow_style=False, sort_keys=False
    ).strip()
    return f"---\n{fm_text}\n---\n{body}"


def _resolve_model(
    agent_path: Path, platform_key: str, batch_model: str | None
) -> str | None:
    content = agent_path.read_text(encoding="utf-8")
    meta, _ = parse_frontmatter(content)
    current_model = str(meta.get("model", ""))

    if batch_model is not None:
        return batch_model

    options = list(MODEL_OPTIONS)
    model_labels = []
    default_idx = 0
    for i, opt in enumerate(options):
        if opt == current_model:
            model_labels.append(f"{opt} ← 当前")
            default_idx = i
        else:
            model_labels.append(opt)

    selected = emoji_radiolist(
        model_labels,
        message=f"为 [{platform_key}] {agent_path.name} 选择 model (↑↓移动, 空格/回车确认, q跳过)",
        default=default_idx,
    )
    if selected is None:
        return None

    chosen = options[model_labels.index(selected)]
    if chosen == "(keep original)":
        return current_model
    if chosen == "(manual input...)":
        manual = input("请输入 model 名称: ").strip()
        return manual if manual else current_model
    return chosen


def _confirm(
    selected: list[tuple[Path, str, str, bool]], sources: dict[str, Path]
) -> bool:
    cwd = Path.cwd()
    rows = []
    for src, cat, src_key, from_harness in selected:
        dst = get_target(src, cat, src_key, from_harness)
        rows.append(_build_table_row(src, cat, _rel_path(dst, cwd)))
    print(f"\n即将拷贝 {len(selected)} 项:")
    _print_table(rows, TABLE_HEADERS)
    return custom_confirm("确认执行？", default=True)


def get_target(
    src_path: Path, category: str, src_key: str = "", from_harness: bool = False
) -> Path:
    cwd = Path.cwd()
    name = src_path.name
    if from_harness:
        if category == "harness":
            return cwd / "custom-harness" / src_key / name
        return cwd / "custom-harness" / src_key / category / name
    if category == "skills":
        return cwd / "skills" / src_key / name
    if category == "config":
        return cwd / "custom-harness" / src_key / name
    return cwd / name


def copy_items(
    selected: list[tuple[Path, str, str, bool]], dry_run: bool = False
) -> None:
    tag = "[DRY RUN] " if dry_run else ""
    targets = []
    for src, cat, src_key, from_harness in selected:
        dst = get_target(src, cat, src_key, from_harness)
        targets.append((src, cat, dst))
        kind = "目录" if src.is_dir() else "文件"
        print(f"{tag}拷贝{kind}: {src} -> {dst}")
        if dry_run:
            continue
        if src.is_dir():
            _remove_path(dst)
            shutil.copytree(src, dst, symlinks=False)
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
    cwd = Path.cwd()
    rows = []
    for src, cat, dst in targets:
        rows.append(_build_table_row(src, cat, _rel_path(dst, cwd)))
    print(f"\n{tag}已拷贝 {len(selected)} 项:")
    _print_table(rows, TABLE_HEADERS)


def scan_install_config(
    sources: dict[str, Path],
) -> list[tuple[Path, Path, str]]:
    cwd = Path.cwd()
    items = []
    for src_key, filename in CONFIG_FILE_MAP.items():
        if src_key not in sources:
            continue
        src = cwd / "custom-harness" / src_key / filename
        if src.exists() and src.is_file():
            dst = sources[src_key] / filename
            items.append((src, dst, src_key))
    return items


def install_config(
    items: list[tuple[Path, Path, str]],
    src_labels: dict[str, str],
    dry_run: bool = False,
) -> None:
    tag = "[DRY RUN] " if dry_run else ""
    installed = 0
    for src, dst, src_key in items:
        new_text = src.read_text(encoding="utf-8")
        if not _check_overwrite(dst, new_text):
            continue
        print(f"{tag}安装配置: {src} -> {dst}")
        if dry_run:
            installed += 1
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)
        installed += 1
    print(f"\n{tag}已安装 {installed} 项配置")


def scan_local_agents(cwd: Path, platform: str) -> list[tuple[Path, str]]:
    """扫描 custom-harness/{platform}/agents/*.md，返回 [(agent_path, platform_key)]"""
    platforms = ["claude", "opencode"] if platform == "all" else [platform]
    agents: list[tuple[Path, str]] = []
    for p in platforms:
        agents_dir = cwd / "custom-harness" / p / "agents"
        if agents_dir.is_dir():
            for md in sorted(agents_dir.glob("*.md")):
                agents.append((md, p))
    return agents


def get_agent_install_target(
    agent_path: Path, platform: str, sources: dict[str, Path]
) -> Path:
    """计算 agent 安装目标路径"""
    return sources[platform] / "custom-harness" / "agents" / agent_path.name


def _run_install(
    sources: dict[str, Path], src_labels: dict[str, str], dry_run: bool = False
) -> None:
    items = scan_install_config(sources)
    if not items:
        print("未发现可安装的配置文件")
        print("  期望路径: custom-harness/claude/CLAUDE.md, custom-harness/opencode/AGENTS.md")
        return
    print("发现可安装的配置文件:")
    for src, dst, src_key in items:
        exists_tag = " [已存在]" if dst.exists() else ""
        print(f"  {src_key}: {src} -> {dst}{exists_tag}")
    print()
    if not custom_confirm("确认安装以上配置？", default=True):
        print("已取消")
        return
    install_config(items, src_labels, dry_run=dry_run)


def _run_install_agents(
    args, sources: dict[str, Path], src_labels: dict[str, str]
) -> None:
    dry_run = args.dry_run
    platform = args.platform
    batch_model = args.model
    cwd = Path.cwd()

    agents = scan_local_agents(cwd, platform)
    if not agents:
        print("未发现可安装的 agents")
        print("  期望路径: custom-harness/{claude,opencode}/agents/*.md")
        return

    label_to_agent = {}
    agent_labels = []
    for agent_path, platform_key in agents:
        label = f"[{platform_key}] {agent_path.name}"
        label_to_agent[label] = (agent_path, platform_key)
        agent_labels.append(label)
    print(f"发现 {len(agents)} 个 agents:")
    for label in agent_labels:
        print(f"  {label}")
    print()

    chosen_labels = emoji_checkbox(
        agent_labels,
        message="选择要安装的 agents (↑↓移动, 空格切换, 回车确认, q退出)",
    )
    if not chosen_labels:
        print("未选择任何 agent")
        return

    agent_models: dict[tuple[Path, str], str] = {}
    for label in chosen_labels:
        agent_path, platform_key = label_to_agent[label]
        model = _resolve_model(agent_path, platform_key, batch_model)
        if model is not None:
            agent_models[(agent_path, platform_key)] = model

    if not agent_models:
        print("没有需要安装的 agent")
        return

    print(f"\n即将安装 {len(agent_models)} 个 agents:")
    rows = []
    for (agent_path, platform_key), model in agent_models.items():
        target = get_agent_install_target(agent_path, platform_key, sources)
        c = CAT_COLORS.get("agents", "")
        rows.append(
            [
                f"{c}{platform_key}{RST}",
                agent_path.name,
                model,
                str(target),
            ]
        )
    _print_table(rows, ["平台", "Agent", "Model", "目标路径"])

    if not custom_confirm("确认安装以上 agents？", default=True):
        print("已取消")
        return

    tag = "[DRY RUN] " if dry_run else ""
    installed = 0
    for (agent_path, platform_key), model in agent_models.items():
        target = get_agent_install_target(agent_path, platform_key, sources)
        content = agent_path.read_text(encoding="utf-8")
        target_content = write_frontmatter(content, {"model": model})

        if not _check_overwrite(target, target_content):
            continue

        print(f"{tag}安装 agent: {agent_path} -> {target} (model: {model})")
        if dry_run:
            installed += 1
            continue

        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(target_content, encoding="utf-8")
        installed += 1

    print(f"\n{tag}已安装 {installed} 个 agents")


def main() -> None:
    _enable_vt100()

    argv = sys.argv[1:]

    # 向后兼容: --install 标志 → install 子命令
    if "--install" in argv:
        argv = [a for a in argv if a != "--install"]
        argv = ["install"] + argv

    # 无子命令时默认 pull (不影响 --help)
    has_subcommand = any(a in argv for a in ("pull", "install", "install-agents"))
    if not has_subcommand and "--help" not in argv and "-h" not in argv:
        argv = ["pull"] + argv

    parser = argparse.ArgumentParser(
        description="交互式拷贝用户域 AI 编码工具配置到当前目录"
    )
    sub = parser.add_subparsers(dest="command")

    pull_p = sub.add_parser("pull", help="拉取模式：从用户域拷贝配置到当前目录")
    pull_p.add_argument("--dry-run", action="store_true", help="预览模式，不实际拷贝")

    install_p = sub.add_parser(
        "install", help="安装模式：从当前目录安装配置到用户域"
    )
    install_p.add_argument(
        "--dry-run", action="store_true", help="预览模式，不实际拷贝"
    )

    ia_p = sub.add_parser(
        "install-agents", help="安装 agents 到用户域并设置 model"
    )
    ia_p.add_argument(
        "--platform",
        choices=["claude", "opencode", "all"],
        default="all",
        help="目标平台 (默认: all)",
    )
    ia_p.add_argument("--model", type=str, default=None, help="批量设置 model")
    ia_p.add_argument(
        "--dry-run", action="store_true", help="预览模式，不实际拷贝"
    )

    args = parser.parse_args(argv)
    sources, src_labels, skip = load_config()

    if args.command == "install":
        _run_install(sources, src_labels, dry_run=args.dry_run)
        return

    if args.command == "install-agents":
        _run_install_agents(args, sources, src_labels)
        return

    # pull (默认)
    print("扫描配置源...")
    categories = scan_sources(sources, skip)
    total = sum(len(v) for v in categories.values())
    if total == 0:
        print("未发现可拷贝的配置项")
        return
    for k, v in categories.items():
        print(f"  {CAT_LABELS.get(k, k.title())}: {len(v)} 项")
    print(f"  共 {total} 项\n")
    selected = interactive_select(categories, sources, src_labels)
    if not selected:
        print("未选择任何项目")
        return
    if not _confirm(selected, sources):
        print("已取消")
        return
    dirs_to_delete: set[str] = set()
    for _src, cat, _src_key, from_harness in selected:
        if from_harness and cat in ("agents", "commands"):
            dirs_to_delete.add(cat)
    for d in sorted(dirs_to_delete):
        target = Path.cwd() / d
        if target.exists():
            _remove_path(target)
            print(f"已删除旧目录: {target}/")
    print(f"\n开始拷贝...\n")
    copy_items(selected, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
