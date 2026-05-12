# Claude Harness — `dispatch` Skill & Subagent Router

Custom harness: consolidate `~/.claude/agents/commands/skills/` under `custom-harness/`; route via explicit `dispatch` skill. Prevents implicit CC auto-discovery; forces named invocation.

## Decision Log

| # | Decision |
|---|---|
| 1 | `agents/` 除 `task-dispatcher.md` 外全部物理迁入 `custom-harness/agents/` |
| 2 | `commands/*.md` → `custom-harness/commands/`; `commands/tools/*` → `custom-harness/bin/` |
| 3 | 执行模型 = 组装器: `dispatch.py` 吐 JSON → Claude 用 `Agent` 工具派生 |
| 4 | Python 先行,后期 Rust 替换 |
| 5 | 触发 = 仅用户显式; CLI = `dispatch <name>` (主) / `dispatch <type>:<name>` (消歧) |
| 6 | Flag: `--help` (含 list) · `--bg` · `--model` |
| 7 | Subagent 策略 = 统一 `general-purpose` + md 正文注入; `tools:` 以软约束注入 prompt |
| 8 | Registry schema = 最简 (`path` + `desc`); skill/agent/command 统一处理 |

## Target Layout

```
~/.claude/
├── agents/
│   └── task-dispatcher.md          # 唯一保留(CC 原生注册)
├── commands/                       # 空(全部迁出)
├── skills/
│   ├── dispatch/SKILL.md           # 新建:核心入口
│   └── graphify/                   # 保留不动
└── custom-harness/
    ├── registry.yaml
    ├── agents/                     # 8 个 agent .md
    ├── commands/                   # dev-flow.md, git-commit.md
    ├── skills/                     # 4 symlink → ~/.agents/skills/*
    └── bin/
        ├── dispatch.py
        └── git_commit_precheck.py
```

## registry.yaml Schema

```yaml
agents:
  bug-fixer:          { path: agents/bug-fixer.md,          desc: "Debug→doc→TDD fix→clean→simplify→review" }
  code-implementer:   { path: agents/code-implementer.md,   desc: "TDD impl for C/C++/Python/Go/TS/Rust" }
  code-reviewer:      { path: agents/code-reviewer.md,      desc: "Review impl against design doc/spec" }
  doc-corrector:      { path: agents/doc-corrector.md,      desc: "Corrects docs to match current code" }
  doc-designer:       { path: agents/doc-designer.md,       desc: "Generates design docs and splits tasks" }
  doc-fixer:          { path: agents/doc-fixer.md,          desc: "Fixes doc vulnerabilities and contradictions" }
  doc-reviewer:       { path: agents/doc-reviewer.md,       desc: "Read-only doc auditor for gaps/risks" }
  git-committer:      { path: agents/git-committer.md,      desc: "Git add+commit w/ gitmoji msg" }
  # ...
commands:
  dev-flow:  { path: commands/dev-flow.md, desc: "..." }
  # ...
skills:
  doc-refine: { path: skills/doc-refine/SKILL.md, desc: "..." }
  # ...
```

Rules:
- 名字跨所有 type 必须唯一
- 同名 → 报错要求 `type:name` 消歧
- 无 `aliases` / `hidden` / `model` 字段(从 md frontmatter 动态读)

## dispatch.py — 核心流程

```
dispatch <name|type:name> <prompt> [--model M] [--bg] [--help [name]]
```

1. 解析 argv
2. `--help` 无参 → 打印总览 + 分组注册项 → 退出
3. `--help <name>` → 打印该项 desc + frontmatter → 退出
4. 加载 `registry.yaml`(定位: `Path(__file__).parents[1]`)
5. 解析名字: 含 `:` → `<type>:<name>` 精确查; 否则全 type 扫, 重名 → exit 2
6. 读 md, 剥 `---...---` frontmatter, 解析 `name/description/model/tools`
7. 组装 prompt(见下)
8. stdout 输出 JSON:

```json
{
  "subagent_type": "general-purpose",
  "description": "<desc 截断 50 字>",
  "prompt": "<组装 prompt>",
  "model": "<frontmatter.model | --model>",
  "run_in_background": true
}
```

> `run_in_background` 仅当 `--bg` 时注入; `model` 缺省时省略。

### Prompt 组装模板

```
You are a one-shot general-purpose subagent executing the definition below.
Follow it literally.

<!-- BEGIN: {type}/{name} -->
{md_body_without_frontmatter}
<!-- END -->

Tool access guidance (soft): originally authored for tools = [{tools}].
Prefer those; avoid unrelated tools.

---
## User Input
{user_prompt}
```

### Exit Codes

| Code | Reason |
|------|--------|
| 0 | 正常(JSON 已输出) |
| 2 | 名字未注册 / 歧义 |
| 3 | md 文件缺失 / frontmatter 解析失败 |
| 4 | registry.yaml 缺失 |

### 依赖

PyYAML(推荐); 无则手写窄范围解析器(registry 结构固定)。

## dispatch/SKILL.md (ultra 压缩)

```markdown
---
name: dispatch
description: User-invoked router. Spawn registered agent/command/skill as one-shot
  subagent. Trigger ONLY on explicit /dispatch from user.
---

# dispatch

## Invoke

User: `/dispatch <name|type:name> <prompt>` [`--model M`] [`--bg`] [`--help`]

## Flow

1. Bash → `python ~/.claude/custom-harness/bin/dispatch.py <argv>`
2. Parse stdout JSON
3. Call Agent tool w/ JSON fields
4. Return subagent result

## `--help`

Forward to dispatch.py. Print stdout verbatim. No subagent spawn.

## Errors

Non-zero exit → print stderr. No retry.

## NEVER

- Don't guess names. Ambiguity → ask user.
- Don't trigger without explicit /dispatch.
- Don't inline registry lookup.
```

## Migration Steps

1. `mkdir -p ~/.claude/custom-harness/{agents,commands,skills,bin}`
2. Move 8 agent `.md` → `custom-harness/agents/` (leave `task-dispatcher.md`)
3. Move `commands/*.md` → `custom-harness/commands/`
4. Move `commands/tools/*` → `custom-harness/bin/`
5. Move 4 skill symlinks → `custom-harness/skills/`
6. Write `registry.yaml`
7. Write `dispatch.py`
8. Create `~/.claude/skills/dispatch/SKILL.md`
9. Fix `git-commit.md` 内路径引用(`commands/tools/` → `custom-harness/bin/`)

## Verification

| Check | Command |
|-------|---------|
| `agents/` 仅剩 task-dispatcher | `ls ~/.claude/agents/` |
| custom-harness 骨架齐 | `ls ~/.claude/custom-harness/{agents,commands,skills,bin}` |
| Symlink 仍指向 `~/.agents/skills/*` | `ls -la ~/.claude/custom-harness/skills/` |
| `--help` 输出 | `python dispatch.py --help` |
| 合法 JSON | `python dispatch.py bug-fixer "test" \| python -m json.tool` |
| skill 型 JSON | `python dispatch.py doc-refine "demo" \| python -m json.tool` |
| 未知名报错 exit 2 | `python dispatch.py nonexistent "x"; echo $?` |

端到端: 新对话键入 `/dispatch bug-fixer "分析虚构 bug"` → Claude 调 Bash → Agent 工具派生 → 看到 bug-fixer workflow 输出。

## Out of Scope (YAGNI)

- aliases / hidden / tags registry 字段
- `--worktree` / `--dry-run` flag
- Rust/Go 重写(后续)
- 自动补全、交互式 picker
- dispatch 日志 / telemetry
- skill `references/` 自动附加

## Risks

| Risk | Mitigation |
|------|------------|
| Windows symlink 移动丢目标 | 删旧 + 重建指向 `~/.agents/skills/*` |
| PyYAML 缺失 | 窄范围手写解析器兜底 |
| `git-commit.md` 旧路径硬编码 | 迁移后 Grep `commands/tools` 并修 |
| 旧 `/dev-flow` 用户习惯 | `--help` 首屏注明"旧 /dev-flow 已失效,改用 /dispatch dev-flow" |
