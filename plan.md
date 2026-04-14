# Sync Config 重构计

## 鹄的

`sync_config.*` → `tools/`；增跨平台启动脚本；`agents/`、`commands/` 按 `src_key` 分子目录。

## 目录（重构后）

```
mycc/
├── tools/
│   ├── sync_config.py
│   └── sync_config.yaml
├── sync.bat
├── sync.sh
├── agents/
│   ├── claude/           # ← ~/.claude/agents/
│   │   ├── bug-fixer.md
│   │   ├── code-implementer.md
│   │   ├── code-reviewer.md
│   │   ├── doc-corrector.md
│   │   ├── doc-designer.md
│   │   ├── doc-reviewer.md
│   │   └── git-committer.md
│   └── opencode/         # ← ~/.config/opencode/agents/
├── commands/
│   ├── claude/           # ← ~/.claude/commands/
│   │   ├── dev-flow.md
│   │   ├── git-commit.md
│   │   └── tools/
│   └── opencode/         # ← ~/.config/opencode/commands/
├── skills/               # 不变 ← ~/.agents/skills/
├── CLAUDE.md             # 不变
├── AGENTS.md             # 不变
└── ...
```

## 步

### 一、建 `tools/`，移入文件

- `sync_config.py` → `tools/sync_config.py`
- `sync_config.yaml` → `tools/sync_config.yaml`

### 二、迁现有文件入子目录

- `agents/*.md`（7）→ `agents/claude/*.md`
- `commands/*`（含 `commands/tools/`）→ `commands/claude/*`

### 三、增启动脚本

#### `sync.bat`（Win11）

```bat
@echo off
python "%~dp0tools\sync_config.py" %*
```

#### `sync.sh`（Debian 12）

```sh
#!/bin/bash
exec python3 "$(dirname "$0")/tools/sync_config.py" "$@"
```

### 四、改 `tools/sync_config.py`

#### a. `scan_sources()` — 增 opencode agents/commands 扫描

```python
elif src_key == "opencode":
    ag = src_root / "AGENTS.md"
    if ag.exists():
        cats["config"].append((ag, src_key))
    for p in _iter_items(src_root / "agents", skip):
        cats["agents"].append((p, src_key))
    for p in _iter_items(src_root / "commands", skip):
        cats["commands"].append((p, src_key))
```

#### b. `interactive_select()` — 保留 `src_key`

- `label_map[label]` → 三元组 `(path, cat_key, src_key)`
- 返 `list[tuple[Path, str, str]]`

#### c. `get_target()` — 增参数 `src_key`，按来源路由

```python
def get_target(src_path: Path, category: str, src_key: str) -> Path:
    cwd = Path.cwd()
    name = src_path.name
    if category in ("agents", "commands"):
        return cwd / category / src_key / name
    if category == "skills":
        return cwd / category / name
    return cwd / name
```

#### d. `_confirm()` + `copy_items()` — 适三元组

- 解包加 `src_key`
- 调 `get_target(src, cat, src_key)`

#### e. `_build_table_row()` — 路径示 `agents/claude/xxx`

### 五、`sync_config.yaml` — 无须改

动态扫描由运行时自检目录存否，yaml 映射不变。

## 不变

- `skills/` ← `~/.agents/skills/`，扫描逻辑不改
- `CLAUDE.md`、`AGENTS.md` 拷贝不改
- `~/.agents` 源不改
