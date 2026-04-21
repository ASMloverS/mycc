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


def emoji_checkbox(
    items: list[str],
    message: str = "选择要拷贝的项目 (↑↓移动, 空格切换, 回车确认, q退出)",
) -> list[str]:
    if not items:
        return []
    cursor = 0
    selected = [False] * len(items)
    visible_rows = max(_term_height() - 4, 5)
    scroll_top = 0
    prev_lines = 0
    while True:
        if cursor < scroll_top:
            scroll_top = cursor
        elif cursor >= scroll_top + visible_rows:
            scroll_top = cursor - visible_rows + 1
        display_count = min(visible_rows, len(items) - scroll_top)
        if prev_lines > 0:
            sys.stdout.write(f"\033[{prev_lines}A")
        sys.stdout.write("\033[J")
        lines = 0
        indicator = ""
        if len(items) > visible_rows:
            indicator = f" ({cursor + 1}/{len(items)})"
        sys.stdout.write(f"\033[1m{message}{indicator}\033[0m\n")
        lines += 1
        for vi in range(display_count):
            i = scroll_top + vi
            mark = "✅" if selected[i] else "⬜"
            pointer = "▶" if i == cursor else " "
            sys.stdout.write(f" {pointer} {mark} {items[i]}\n")
            lines += 1
        if len(items) > visible_rows:
            bar_total = visible_rows
            bar_pos = int(bar_total * cursor / max(len(items) - 1, 1))
            bar_chars = ["─"] * bar_total
            bar_chars[bar_pos] = "█"
            sys.stdout.write(f"  {''.join(bar_chars)}\n")
            lines += 1
        sys.stdout.flush()
        prev_lines = lines
        key = get_key()
        if key == "up":
            cursor = (cursor - 1) % len(items)
        elif key == "down":
            cursor = (cursor + 1) % len(items)
        elif key == "space":
            selected[cursor] = not selected[cursor]
        elif key == "enter":
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            return [items[i] for i in range(len(items)) if selected[i]]
        elif key == "q":
            sys.stdout.write(f"\033[{prev_lines}A\033[J\n")
            return []


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
        if src_key == "claude":
            fp = src_root / "CLAUDE.md"
            if fp.exists():
                cats.setdefault("config", []).append((fp, src_key, "", False))
        elif src_key == "opencode":
            ag = src_root / "AGENTS.md"
            if ag.exists():
                cats.setdefault("config", []).append((ag, src_key, "", False))
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
    w = 0
    for ch in s:
        eaw = unicodedata.east_asian_width(ch)
        w += 2 if eaw in ("W", "F") else 1
    return w


TABLE_HEADERS = ["类别", "名称", "类型", "目标路径"]


def _build_table_row(src: Path, cat: str, rel_dst: str) -> list[str]:
    c = CAT_COLORS.get(cat, "")
    return [
        f"{c}{CAT_LABELS[cat]}{RST}",
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


def _confirm(
    selected: list[tuple[Path, str, str, bool]], sources: dict[str, Path]
) -> bool:
    cwd = Path.cwd()
    rows = []
    for src, cat, src_key, from_harness in selected:
        dst = get_target(src, cat, src_key, from_harness)
        try:
            rel_dst = str(dst.relative_to(cwd))
        except ValueError:
            rel_dst = str(dst)
        rows.append(_build_table_row(src, cat, rel_dst))
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
            if dst.exists():
                if dst.is_dir():
                    shutil.rmtree(dst)
                else:
                    dst.unlink()
            shutil.copytree(src, dst, symlinks=False)
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
    cwd = Path.cwd()
    rows = []
    for src, cat, dst in targets:
        try:
            rel_dst = str(dst.relative_to(cwd))
        except ValueError:
            rel_dst = str(dst)
        rows.append(_build_table_row(src, cat, rel_dst))
    print(f"\n{tag}已拷贝 {len(selected)} 项:")
    _print_table(rows, TABLE_HEADERS)


def main() -> None:
    _enable_vt100()
    parser = argparse.ArgumentParser(
        description="交互式拷贝用户域 AI 编码工具配置到当前目录"
    )
    parser.add_argument("--dry-run", action="store_true", help="预览模式，不实际拷贝")
    args = parser.parse_args()
    sources, src_labels, skip = load_config()
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
            if target.is_dir():
                shutil.rmtree(target)
            else:
                target.unlink()
            print(f"已删除旧目录: {target}/")
    print(f"\n开始拷贝...\n")
    copy_items(selected, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
