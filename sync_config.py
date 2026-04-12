#!/usr/bin/env python3
"""sync_config.py - 交互式拷贝用户域 AI 编码工具配置到当前目录"""

import argparse
import datetime
import os
import shutil
import sys
from pathlib import Path

try:
    import yaml
except ImportError:
    yaml = None

HOME = Path.home()

DEFAULT_SOURCES = {
    "claude": HOME / ".claude",
    "agents": HOME / ".agents",
    "opencode": HOME / ".config" / "opencode",
}
DEFAULT_SRC_LABELS = {
    "claude": "~/.claude",
    "agents": "~/.agents",
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
    "config": "Config",
}
CAT_COLORS = {
    "agents": "\033[36m",
    "commands": "\033[33m",
    "skills": "\033[35m",
    "config": "\033[32m",
}
RST = "\033[0m"


def _enable_vt100():
    if sys.platform != "win32":
        return
    import ctypes

    k = ctypes.windll.kernel32
    h = k.GetStdHandle(-11)
    m = ctypes.c_ulong()
    k.GetConsoleMode(h, ctypes.byref(m))
    k.SetConsoleMode(h, m.value | 0x0004)


def load_config():
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


def _get_key_windows():
    import msvcrt

    ch = msvcrt.getwch()
    if ch in ("\x00", "\xe0"):
        ch2 = msvcrt.getwch()
        if ch2 == "H":
            return "up"
        if ch2 == "P":
            return "down"
        if ch2 == "K":
            return "left"
        if ch2 == "M":
            return "right"
        return f"special:{ch2}"
    if ch == "\r":
        return "enter"
    if ch == " ":
        return "space"
    if ch in ("q", "Q"):
        return "q"
    if ch in ("y", "Y"):
        return "y"
    if ch in ("n", "N"):
        return "n"
    return ch


def _get_key_linux():
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
                if ch3 == "A":
                    return "up"
                if ch3 == "B":
                    return "down"
                if ch3 == "D":
                    return "left"
                if ch3 == "C":
                    return "right"
                return f"esc[{ch3}"
            return f"esc:{ch2}"
        if ch in ("\r", "\n"):
            return "enter"
        if ch == " ":
            return "space"
        if ch in ("q", "Q"):
            return "q"
        if ch in ("y", "Y"):
            return "y"
        if ch in ("n", "N"):
            return "n"
        return ch
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)


get_key = _get_key_windows if sys.platform == "win32" else _get_key_linux


# --- Emoji Checkbox ---


def _term_height():
    try:
        return os.get_terminal_size().lines
    except OSError:
        return 24


def _cur_pos():
    sys.stdout.write("\033[6n")
    sys.stdout.flush()
    buf = ""
    while True:
        ch = sys.stdin.read(1)
        buf += ch
        if ch == "R":
            break
    try:
        rows, cols = buf.lstrip("\x1b[").rstrip("R").split(";")
        return int(rows), int(cols)
    except Exception:
        return None, None


def _get_cursor_row():
    if sys.platform == "win32":
        import ctypes

        h = ctypes.windll.kernel32.GetStdHandle(-11)
        csbi = ctypes.create_string_buffer(22)
        ctypes.windll.kernel32.GetConsoleScreenBufferInfo(h, csbi)
        row = int.from_bytes(csbi.raw[4:8], "little")
        return row
    try:
        import termios

        fd = sys.stdin.fileno()
        old = termios.tcgetattr(fd)
        try:
            import tty

            tty.setraw(fd)
            row, _ = _cur_pos()
            return row
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old)
    except Exception:
        return None


def emoji_checkbox(
    items, message="选择要拷贝的项目 (↑↓移动, 空格切换, 回车确认, q退出)"
):
    if not items:
        return []
    cursor = 0
    selected = [False] * len(items)
    visible_rows = max(_term_height() - 4, 5)
    scroll_top = 0
    prev_lines = 0
    anchor_row = _get_cursor_row()
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


def custom_confirm(message="确认执行？", default=True):
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


def _iter_items(directory, skip):
    if not directory.exists():
        return
    for p in directory.iterdir():
        if p.name in skip or p.name.startswith("."):
            continue
        yield p


def scan_sources(sources, skip):
    cats = {"agents": [], "commands": [], "skills": [], "config": []}
    for src_key, src_root in sources.items():
        if not src_root.exists():
            continue
        if src_key == "claude":
            for p in _iter_items(src_root / "agents", skip):
                cats["agents"].append((p, src_key))
            for p in _iter_items(src_root / "commands", skip):
                cats["commands"].append((p, src_key))
            fp = src_root / "CLAUDE.md"
            if fp.exists():
                cats["config"].append((fp, src_key))
        elif src_key == "agents":
            sk_dir = src_root / "skills"
            if sk_dir.exists():
                for p in _iter_items(sk_dir, skip):
                    cats["skills"].append((p, src_key))
        elif src_key == "opencode":
            ag = src_root / "AGENTS.md"
            if ag.exists():
                cats["config"].append((ag, src_key))
    return cats


def _fmt_size(n):
    for unit in ("B", "KB", "MB", "GB"):
        if n < 1024:
            return f"{n:.0f}{unit}" if unit == "B" else f"{n:.1f}{unit}"
        n /= 1024
    return f"{n:.1f}TB"


def _item_detail(path, src_key, sources, src_labels):
    kind = "Dir" if path.is_dir() else "File"
    if path.is_dir():
        total = sum(f.stat().st_size for f in path.rglob("*") if f.is_file())
    else:
        total = path.stat().st_size
    size = _fmt_size(total)
    mtime = datetime.datetime.fromtimestamp(path.stat().st_mtime).strftime(
        "%Y-%m-%d %H:%M"
    )
    rel = path.relative_to(sources[src_key])
    return rel, kind, size, mtime


def interactive_select(categories, sources, src_labels):
    all_labels = []
    label_map = {}
    for cat_key in ("agents", "commands", "skills", "config"):
        items = categories.get(cat_key, [])
        if not items:
            continue
        c = CAT_COLORS.get(cat_key, "")
        for path, src_key in items:
            rel, kind, size, mtime = _item_detail(path, src_key, sources, src_labels)
            tag = f"{c}[{CAT_LABELS[cat_key]}]{RST}"
            label = f"{tag} {rel}  [{kind}]  {size}  {mtime}  ({src_labels[src_key]})"
            all_labels.append(label)
            label_map[label] = (path, cat_key)
    if not all_labels:
        return []
    chosen = emoji_checkbox(all_labels)
    return [label_map[c] for c in chosen]


def _strip_ansi(s):
    import re

    return re.sub(r"\033\[[0-9;]*m", "", s)


def _print_table(rows, headers):
    col_count = len(headers)
    widths = [len(h) for h in headers]
    stripped = []
    for row in rows:
        srow = []
        for j, cell in enumerate(row):
            sc = _strip_ansi(cell)
            srow.append(sc)
            widths[j] = max(widths[j], len(sc))
        stripped.append(srow)
    sep = "┼".join("─" * (w + 2) for w in widths)
    top = "┌" + "┬".join("─" * (w + 2) for w in widths) + "┐"
    bot = "└" + "┴".join("─" * (w + 2) for w in widths) + "┘"
    mid = "├" + sep + "┤"

    def fmt(cells, colored=None):
        parts = []
        for j, sc in enumerate(cells):
            pad = widths[j] - len(sc)
            if colored and j < len(colored):
                parts.append(f" {colored[j]}{' ' * pad}{RST} ")
            else:
                parts.append(f" {sc}{' ' * pad} ")
        return "│" + "│".join(parts) + "│"

    print(f"\n  {top}")
    print(f"  {fmt(headers)}")
    print(f"  {mid}")
    for i, srow in enumerate(stripped):
        print(f"  {fmt(srow, rows[i])}")
    print(f"  {bot}")


def _confirm(selected, sources):
    cwd = Path.cwd()
    rows = []
    for src, cat in selected:
        dst = get_target(src, cat)
        try:
            rel_dst = str(dst.relative_to(cwd))
        except ValueError:
            rel_dst = str(dst)
        c = CAT_COLORS.get(cat, "")
        rows.append(
            [
                f"{c}{CAT_LABELS[cat]}{RST}",
                str(src.name),
                "Dir" if src.is_dir() else "File",
                rel_dst,
            ]
        )
    print(f"\n即将拷贝 {len(selected)} 项:")
    _print_table(rows, ["Category", "Name", "Type", "Target"])
    return custom_confirm("确认执行？", default=True)


def get_target(src_path, category):
    cwd = Path.cwd()
    name = src_path.name
    if category in ("agents", "commands", "skills"):
        return cwd / category / name
    return cwd / name


def copy_items(selected, dry_run=False):
    tag = "[DRY RUN] " if dry_run else ""
    targets = []
    for src, cat in selected:
        dst = get_target(src, cat)
        targets.append((src, cat, dst))
        kind = "Dir" if src.is_dir() else "File"
        print(f"{tag}拷贝{kind}: {src} -> {dst}")
        if dry_run:
            continue
        if src.is_dir():
            if dst.exists():
                shutil.rmtree(dst)
            shutil.copytree(src, dst)
        else:
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
    cwd = Path.cwd()
    rows = []
    for src, cat, dst in targets:
        c = CAT_COLORS.get(cat, "")
        try:
            rel_dst = str(dst.relative_to(cwd))
        except ValueError:
            rel_dst = str(dst)
        rows.append(
            [
                f"{c}{CAT_LABELS[cat]}{RST}",
                str(src.name),
                "Dir" if src.is_dir() else "File",
                rel_dst,
            ]
        )
    print(f"\n{tag}已拷贝 {len(selected)} 项:")
    _print_table(rows, ["Category", "Name", "Type", "Target"])


def main():
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
        print(f"  {CAT_LABELS[k]}: {len(v)} 项")
    print(f"  共 {total} 项\n")
    selected = interactive_select(categories, sources, src_labels)
    if not selected:
        print("未选择任何项目")
        return
    if not _confirm(selected, sources):
        print("已取消")
        return
    print(f"\n开始拷贝...\n")
    copy_items(selected, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
