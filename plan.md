# Sync Config 重构计

## 鹄的

`sync_config.*` → `tools/`；增跨平台启动脚本；立 `custom-harness` 机制；pull/install 双向同步；install-agents 带 model 选。

## 目次

```
mycc/
├── tools/
│   ├── sync_config.py          # 875 行
│   ├── sync_config.yaml        # 可选配置
│   └── test_sync_config.py     # pytest 509 行
├── sync.bat / sync.sh          # 跨平台启动
├── custom-harness/
│   ├── claude/{agents,commands,bin,CLAUDE.md}
│   └── opencode/{agents,commands,AGENTS.md}
├── skills/claude/              # ← ~/.agents/skills/
└── ...
```

## 已成

### 一、tools/ 迁移 ✅

三文件就位：主脚本、YAML 配置、pytest 测试。

### 二、启动脚本 ✅

`sync.bat` → `python "%~dp0tools\sync_config.py" %*`
`sync.sh` → `exec python3 "$(dirname "$0")/tools/sync_config.py" "$@"`

### 三、harness 机制 ✅

源端 `~/.{claude,config/opencode}/custom-harness/{agents,commands,bin}/` → 目标端 `custom-harness/{src_key}/{category}/`。

### 四、子命令 ✅

| 子命令 | 方向 | 用途 |
|--------|------|------|
| `pull`（默认） | 用户域 → 项目 | 扫+选+拷 |
| `install` | 项目 → 用户域 | 装 CLAUDE.md/AGENTS.md |
| `install-agents` | 项目 → 用户域 | 装 agents + 设 model |

`--install` 向后兼容 → 映射为 `install`。

### 五、核心

**四元组**：`(path, category, src_key, from_harness)`

**扫描**：
- `_scan_harness()` — 子目录→分类 `from_harness=True`；顶层→`harness`；`skills/`→`skills` `from_harness=False`
- `scan_sources()` — 遍源 → harness + config（CLAUDE.md/AGENTS.md）

**路由 `get_target()`**：

| 条件 | 目标 |
|------|------|
| `from_harness ∧ category=="harness"` | `custom-harness/{src_key}/{name}` |
| `from_harness` | `custom-harness/{src_key}/{category}/{name}` |
| `category=="skills"` | `skills/{src_key}/{name}` |
| `category=="config"` | `custom-harness/{src_key}/{name}` |
| 余 | `{name}` |

**Frontmatter**：`parse_frontmatter` → `(meta, body)`；`write_frontmatter` → 改 YAML 字段；`_resolve_model()` — radio 选 model。

**install**：`scan_install_config()` 扫 `custom-harness/{platform}/{config}` → `install_config()` 带 `_check_overwrite`（同→跳、异→确认）。

**install-agents**：`scan_local_agents()` → 逐 agent 选 model → 写 frontmatter → 装至用户域。`--platform` + `--model` 批设。

**UI**：`_interactive_list()` 统一 checkbox/radio + 滚动条；`emoji_checkbox`/`emoji_radiolist`/`custom_confirm`；`_print_table()` ANSI 彩色表（UTF-8 宽度感知）；跨平台键盘（Win `msvcrt` / Linux `termios`）。

**余**：`--dry-run` 三子命令通用；`load_config()` YAML 可覆源/跳过；拷前删旧 `agents/`、`commands/`；示大小 + 改时；`_enable_vt100` Win VT100。

### 六、测试 ✅

`TestIterItems`（过滤）、`TestScanSources`（扫描+标记）、`TestGetTarget`（六路由）、`TestBuildTableRow`、`TestCopyItems`（dry-run/实拷）、`TestEndToEnd`（全流程）、`TestScanInstallConfig`（四场景）、`TestInstallConfig`（五行为）。

### 七、sync_config.yaml ✅

默认源 + 跳过列表，与代码一致。

## 不变

`~/.agents` 源不改。
