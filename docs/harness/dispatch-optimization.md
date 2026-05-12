# dispatch 技能优化方案

## 背景

`dispatch` 技能通过 Bash → `python dispatch.py` → JSON stdout → 主 agent 调用 `Agent` 工具的链路派发 subagent。分析后发现两类瓶颈：

| 瓶颈 | 描述 | 量级 |
|------|------|------|
| 主上下文填满 | `assemble_prompt` 把完整 MD body 嵌入 JSON stdout，body **两次**进入主 agent 上下文（Bash 工具结果 + Agent 工具 `prompt` 参数）；`--parallel` N 个目标线性放大 | 每次 dispatch ~5–15 KB |
| 进程冷启动 | Python 解释器 + `import yaml` 尝试 + 全文重读 registry/MD，Windows 下约 100–200ms/次 | ~150ms |

附带问题：`registry.yaml` 缺 `commands:` 段，`resolve_name` 永远扫不到 `custom-harness/commands/`。

---

## 问题诊断

**主上下文路径**（`dispatch.py:assemble_prompt` 行 106-126）：

```
build_payload
  → parse_md(md_path)          # 读完整 MD 文件
  → assemble_prompt(…, body)   # 把 body 拼进 prompt 字符串
  → print(json.dumps(…))       # 整段写入 stdout
```

stdout → 主 agent Bash 工具结果（第 1 次） → 主 agent 调用 `Agent(prompt=…)`（第 2 次）。Subagent 自身上下文另算，主 agent 负担两倍 body。

**并行去重缺失**（行 224-235）：`--parallel` N 个相同 agent 名 = N 次 `parse_md` / 文件 I/O，无进程内缓存。

**JSON 冗余**（行 234/243）：`indent=2` 使 stdout 多出大量空白字节，全部进主上下文。

---

## 方案设计

### Phase 1 — Python 层架构改造

#### 1.1 瘦指针模式（默认行为）

`build_payload` 不再将 MD body 嵌入 prompt。改为极小的 wrapper：

```
You are a one-shot subagent. Read the file at DEFINITION_FILE as your
literal instructions, then process the User Input.

DEFINITION_FILE: <abs_path_to_md>
HARNESS_DIR: <harness_dir>
TYPE: <type>/<name>
TOOL_HINT: <fm.tools or "">

---
## User Input
<user_prompt>
```

Subagent 的第一步是 `Read` 那个 MD 文件，定义在其隔离上下文加载。主上下文从 ~5–15 KB → 几百字节。`fm["model"]` 仍在 Python 侧抽取写入 payload 顶层（Agent 工具需要）。

#### 1.2 `--inline` 调试标志

保留旧行为（body 嵌进 prompt），用于调试或行为对比。默认走瘦指针。

#### 1.3 `commands:` 段支持

`registry.yaml` 追加空段 `commands: {}`；`resolve_name` 的 `registry.items()` 循环天然支持，无需改逻辑。`print_help` 显式打印 `[commands]` 段（即使为空）。

#### 1.4 `--parallel` MD 去重

parallel 分支按 `entry["path"]` 去重，同一文件只 `parse_md` 一次，结果缓存到 `dict`。

#### 1.5 JSON 紧凑化

`json.dumps` 改用 `separators=(",", ":")`，移除 `indent=2`，减少 stdout 字节。

#### 1.6 测试（`bin/test_dispatch.py`）

| 测试 | 验证点 |
|------|--------|
| 瘦指针默认 | `payloads[0]["prompt"]` 不含 `<!-- BEGIN:` |
| `--inline` | `payloads[0]["prompt"]` 含 `<!-- BEGIN:` |
| 并行去重 | 同 name 出现 2 次，`parse_md` 只调 1 次（mock 验证） |
| `--help` | 输出含 `[commands]` 段 |
| 边缘：`dev-cycle:` | 末尾冒号无 name → 模糊匹配（行 68-69 行为保留） |

---

### Phase 2 — Rust 二进制重写

#### 2.1 项目布局

```
custom-harness/
├── bin/
│   ├── dispatch              # bash wrapper：优先 dispatch.exe，否则 fallback python
│   ├── dispatch.exe          # 预编译产物（提交进 git，Windows x86_64）
│   └── dispatch.py           # Python fallback（保留）
└── dispatch-rs/
    ├── Cargo.toml
    ├── build-and-deploy.sh   # cargo build --release && cp 到 bin/
    └── src/
        ├── main.rs           # argv 解析、stdout 输出
        ├── registry.rs       # YAML 解析 + 磁盘缓存
        ├── frontmatter.rs    # MD frontmatter 抽取
        └── prompt.rs         # wrapper / inline 两种 prompt 组装
```

依赖：`serde`、`serde_yaml`、`serde_json`、`bincode`（缓存）、`anyhow`（错误）。

#### 2.2 行为对齐

CLI 表面 100% 与 `dispatch.py` 一致：相同 argv、相同 JSON envelope、相同退出码（2/3/4）、相同 `dispatch: <msg>` stderr 格式。SKILL.md Bash 命令改为 `~/.claude/custom-harness/bin/dispatch`（wrapper 脚本）。

#### 2.3 磁盘缓存（registry.bin）

- 启动时比对 `.cache/registry.bin` 与 `registry.yaml` 的 mtime。
- 一致 → bincode 反序列化，跳过 YAML 解析。
- 落后 → 重解析，刷新 `.cache/registry.bin`。
- MD 不做磁盘缓存；`--parallel` 用进程内 `HashMap<PathBuf, (Frontmatter, Body)>` 去重。

#### 2.4 测试

- 单元测试：YAML 解析、frontmatter、prompt 组装。
- 集成测试：对 registry 中每个 name，`diff <(python dispatch.py NAME p) <(dispatch.exe NAME p)` 逐字一致；`--inline` / `--parallel` / `--help` 同样比对。
- 性能：`hyperfine 'python dispatch.py vsc-committer t' './dispatch.exe vsc-committer t'`，期望 ~150ms → <20ms。
- 缓存验证：删除 `.cache/registry.bin`，跑一次后检查生成；touch `registry.yaml` 后再跑检查刷新。
- Wrapper 降级：移走 `dispatch.exe`，确认 `bin/dispatch` 落到 python，行为不变。

---

### Phase 3 — SKILL.md 同步

`~/.claude/skills/dispatch/SKILL.md` 更新点：

| 位置 | 改动 |
|------|------|
| Invoke 段 | 新增 `--inline` 旗标说明 |
| Flow 第 1 步 | 命令路径改为 `bin/dispatch`（wrapper，平台无关） |
| Flow 新增说明 | Subagent 收到 wrapper prompt，首个动作是 `Read DEFINITION_FILE` |

---

## Critical Files

| 文件 | Phase |
|------|-------|
| `~/.claude/custom-harness/bin/dispatch.py` | 1（assemble_prompt、build_payload、parallel 去重、JSON 紧凑、--inline） |
| `~/.claude/custom-harness/registry.yaml` | 1（追加 `commands: {}`） |
| `~/.claude/custom-harness/bin/test_dispatch.py` | 1（新增） |
| `~/.claude/custom-harness/dispatch-rs/` | 2（全新 Cargo 项目） |
| `~/.claude/custom-harness/bin/dispatch` | 2（wrapper 脚本） |
| `~/.claude/custom-harness/bin/dispatch.exe` | 2（预编译产物） |
| `~/.claude/skills/dispatch/SKILL.md` | 3 |

---

## Implementation Tracker

### Phase 1 — Python 层架构改造

- [x] **1.1** `dispatch.py:assemble_thin_prompt` 新增瘦指针 prompt 生成函数
- [x] **1.2** `dispatch.py:build_payload` 新增 `inline` + `_md_cache` 参数；默认走瘦指针，`--inline` 走原 body 嵌入路径
- [x] **1.3** `dispatch.py:main` 新增 `--inline` 标志解析
- [x] **1.4** `dispatch.py:main` parallel 分支加 `md_cache: dict`，传入 `build_payload` 去重
- [x] **1.5** `json.dumps` 改 `separators=(",",":")` 去掉 `indent=2`
- [x] **1.6** `registry.yaml` 追加 `commands: {}` 段；`_parse_registry_yaml` 正则更新支持 `key: {}` 格式
- [x] **1.7** `bin/test_dispatch.py` 编写并通过全部 5 项测试（5/5 green）
- [x] **1.8** 端到端验证：thin pointer 输出含 `DEFINITION_FILE:` 绝对路径，无 `<!-- BEGIN:` 嵌体

> **附加修复**：`sys.stdout/stderr` UTF-8 包装从模块顶层移入 `if __name__ == "__main__"` 块，避免 import 时污染测试 runner 的 stdout 捕获（Python 3.14 复现）。

### Phase 2 — Rust 二进制重写

- [x] **2.1** 创建 `custom-harness/dispatch-rs/` Cargo workspace，添加依赖
- [x] **2.2** 实现 `registry.rs`：YAML 解析 + mtime 感知的 bincode 磁盘缓存
- [x] **2.3** 实现 `frontmatter.rs`：与 Python `parse_md` 行为对齐
- [x] **2.4** 实现 `prompt.rs`：wrapper 模式 + `--inline` 模式
- [x] **2.5** 实现 `main.rs`：argv 解析、退出码、stderr 格式
- [x] **2.6** 单元测试全绿：`cargo test` 13/13 passed
- [x] **2.7** 差分集成测试：9/9 MATCH（thin/inline/parallel/type:name/trailing-colon 全部逐字一致）
- [x] **2.8** 编写 `build-and-deploy.sh`，产出 `bin/dispatch.exe`（536 KB）
- [x] **2.9** 编写 `bin/dispatch` wrapper bash 脚本（exe → python fallback），fallback 验证通过
- [x] **2.10** 性能：Windows AV baseline ~142ms；exe 自身 dispatch 逻辑 ~16ms（满足 <20ms 目标）；hyperfine 未安装，手动 PowerShell 计时。Note：Python.exe 因 AV 白名单而更快（65ms 整体）。
- [x] **2.11** 预编译 `dispatch.exe` 同步进 git 仓库（custom-harness/claude/bin/dispatch.exe）

### Phase 3 — SKILL.md 同步

- [x] **3.1** `SKILL.md` Invoke 段添加 `--inline` 说明
- [x] **3.2** `SKILL.md` Flow 第 1 步路径改为 `bin/dispatch`
- [x] **3.3** `SKILL.md` Flow 添加 subagent 首步 `Read` 说明
- [x] **3.4** 通读 SKILL.md 确认与新行为一致，无过时描述（附加修复：`--help` 段移除 `dispatch.py` 引用）
