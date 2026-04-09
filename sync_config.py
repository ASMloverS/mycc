#!/usr/bin/env python3
"""sync_config.py - 交互式拷贝用户域 AI 编码工具配置到当前目录"""

import argparse
import shutil
import sys
from pathlib import Path

try:
    import inquirer
except ImportError:
    print("需要 inquirer 库: pip install inquirer")
    sys.exit(1)

HOME = Path.home()
SOURCES = {
    "claude": HOME / ".claude",
    "agents": HOME / ".agents",
    "opencode": HOME / ".config" / "opencode",
}
SRC_LABELS = {
    "claude": "~/.claude",
    "agents": "~/.agents",
    "opencode": "~/.config/opencode",
}
SKIP = {
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
    "config": "配置文件",
}


def _iter_items(directory):
    if not directory.exists():
        return
    for p in directory.iterdir():
        if p.name in SKIP or p.name.startswith("."):
            continue
        yield p


def scan_sources():
    cats = {"agents": [], "commands": [], "skills": [], "config": []}
    for src_key, src_root in SOURCES.items():
        if not src_root.exists():
            continue
        if src_key == "claude":
            for p in _iter_items(src_root / "agents"):
                cats["agents"].append((p, src_key))
            for p in _iter_items(src_root / "commands"):
                cats["commands"].append((p, src_key))
            for name in ["CLAUDE.md", "settings.json"]:
                fp = src_root / name
                if fp.exists() and fp.name not in SKIP:
                    cats["config"].append((fp, src_key))
        sk_dir = src_root / "skills"
        if sk_dir.exists():
            for p in _iter_items(sk_dir):
                cats["skills"].append((p, src_key))
        if src_key == "opencode":
            for p in src_root.glob("opencode*.json"):
                if p.name not in SKIP:
                    cats["config"].append((p, src_key))
            pkg = src_root / "package.json"
            if pkg.exists():
                cats["config"].append((pkg, src_key))
    return cats


def interactive_select(categories):
    selected = []
    for cat_key, items in categories.items():
        if not items:
            continue
        choices = []
        for path, src_key in items:
            rel = path.relative_to(SOURCES[src_key])
            choices.append(f"{rel}  ({SRC_LABELS[src_key]})")
        questions = [
            inquirer.Checkbox(
                "items", message=f"选择要拷贝的 {CAT_LABELS[cat_key]}", choices=choices
            )
        ]
        answers = inquirer.prompt(questions)
        if not answers or not answers["items"]:
            continue
        for chosen in answers["items"]:
            idx = choices.index(chosen)
            selected.append((items[idx][0], cat_key))
    return selected


def get_target(src_path, category):
    cwd = Path.cwd()
    name = src_path.name
    if category in ("agents", "commands", "skills"):
        return cwd / category / name
    return cwd / name


def copy_items(selected, dry_run=False):
    copied = 0
    for src, cat in selected:
        dst = get_target(src, cat)
        tag = "[DRY RUN] " if dry_run else ""
        if src.is_dir():
            print(f"{tag}拷贝目录: {src} -> {dst}")
            if not dry_run:
                if dst.exists():
                    shutil.rmtree(dst)
                shutil.copytree(src, dst)
        else:
            print(f"{tag}拷贝文件: {src} -> {dst}")
            if not dry_run:
                dst.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(src, dst)
        copied += 1
    print(f"\n{'[DRY RUN] ' if dry_run else ''}完成: {copied} 项")


def main():
    parser = argparse.ArgumentParser(
        description="交互式拷贝用户域 AI 编码工具配置到当前目录"
    )
    parser.add_argument("--dry-run", action="store_true", help="预览模式，不实际拷贝")
    args = parser.parse_args()
    print("扫描配置源...")
    categories = scan_sources()
    total = sum(len(v) for v in categories.values())
    if total == 0:
        print("未发现可拷贝的配置项")
        return
    for k, v in categories.items():
        print(f"  {CAT_LABELS[k]}: {len(v)} 项")
    print(f"  共 {total} 项\n")
    selected = interactive_select(categories)
    if not selected:
        print("未选择任何项目")
        return
    print(f"\n已选择 {len(selected)} 项，开始拷贝...\n")
    copy_items(selected, dry_run=args.dry_run)


if __name__ == "__main__":
    main()
