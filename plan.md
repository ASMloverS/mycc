# Sync Config 重构计

## 鹄的

`sync_config.*` → `tools/`；增跨平台启动脚本；引入 `custom-harness` 机制，agents/commands/bin/skills 按源分目录同步。

## 目录（现状）

```
mycc/
├── tools/
│   ├── sync_config.py          # 主脚本
│   └── sync_config.yaml        # 可选配置（源路径、跳过列表）
├── sync.bat                    # Win11 启动
├── sync.sh                     # Debian 12 启动
├── custom-harness/
│   └── claude/                 # ← harness 源：~/.claude/custom-harness/
│       ├── agents/             # agent markdown 文件
│       ├── commands/           # command markdown 文件
│       ├── bin/                # 辅助脚本
│       ├── skills/             # harness 内 skills
│       └── registry.yaml
├── skills/                     # ← ~/.agents/skills/ (非 harness)
├── CLAUDE.md                   # ← ~/.claude/CLAUDE.md
├── AGENTS.md                   # ← ~/.config/opencode/AGENTS.md
└── ...
```

## 已完成

### 一、`tools/` 目录 + 文件迁移 ✅

- `tools/sync_config.py` — 主脚本（548 行）
- `tools/sync_config.yaml` — 可选 YAML 配置

### 二、跨平台启动脚本 ✅

- `sync.bat` — `python "%~dp0tools\sync_config.py" %*`
- `sync.sh` — `exec python3 "$(dirname "$0")/tools/sync_config.py" "$@"`

### 三、harness 机制替代原 agents/commands 子目录方案 ✅

原方案：`agents/claude/*.md`、`commands/claude/*` 按源分目录。
实际方案：引入 `custom-harness` 概念，源端按 `~/.claude/custom-harness/{agents,commands,bin,skills}/` 组织，目标端拷贝至 `custom-harness/{src_key}/{category}/`。

### 四、核心实现细节

#### 数据模型：四元组

```python
(path: Path, category: str, src_key: str, from_harness: bool)
```

`from_harness=True` 标识来自源端 `custom-harness/` 子目录的项目。

#### a. `_scan_harness()` — harness 扫描

扫描 `{src_root}/custom-harness/`：
- 子目录（agents/commands/bin 等）→ 按子目录名分类
- 顶层文件 → 归入 `harness` 类
- 额外扫描 `{src_root}/skills/` → 归入 `skills`

#### b. `scan_sources()` — 总入口

```python
for src_key, src_root in sources:
    _scan_harness(src_root, src_key, skip, cats)
    # claude: CLAUDE.md → config
    # opencode: AGENTS.md → config
```

不直接扫描 opencode 的 agents/commands，统一走 harness 路径。

#### c. `get_target()` — 按来源路由

```python
def get_target(src_path, category, src_key="", from_harness=False):
    if from_harness:
        if category == "harness":
            return cwd / "custom-harness" / src_key / name
        return cwd / "custom-harness" / src_key / category / name
    if category == "skills":
        return cwd / "skills" / src_key / name
    return cwd / name
```

#### d. 交互 UI

- `emoji_checkbox()` — 上下移动、空格切换、回车确认，含滚动条
- `custom_confirm()` — 左右切换确认/取消
- `_print_table()` — 类别/名称/类型/目标路径 表格
- 跨平台键盘输入（Windows `msvcrt` / Linux `termios`）
- Windows VT100 转义启用（`_enable_vt100`）

#### e. 其他特性

- `--dry-run` 预览模式
- YAML 配置加载（`load_config`），可选覆盖默认源和跳过列表
- 拷贝前自动删除旧 `agents/`、`commands/` 目录（`main` 尾段）
- 显示文件大小（`_fmt_size`）和修改时间

### 五、`sync_config.yaml` ✅

默认源路径 + 跳过列表，与 `DEFAULT_SOURCES`/`DEFAULT_SKIP` 一致。无需改动。

## 不变

- `CLAUDE.md`、`AGENTS.md` 拷贝不改
- `~/.agents` 源不改
