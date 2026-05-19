# 阶段 3：渐进式披露 — CLAUDE.md 瘦身

## 目标

将 CLAUDE.md 从"什么都塞"变为"入口 + 指针"。只保留每次会话都需要的核心规则，其余按需加载。

## 原则

来源参考：[Skill best practices — Progressive disclosure](https://code.claude.com/docs/en/agents-and-tools/agent-skills/best-practices)

1. **CLAUDE.md 只放每次会话都需要的内容** — 其他一切通过指针引用
2. **不要为 Claude 已知的事情浪费 token** — Claude 天然理解 bash、git、常见框架
3. **分层加载** — 根目录放全局，子目录放局部，Auto Memory 放经验，skill 放专业领域

## 当前 CLAUDE.md 分析

当前 `~/.claude/CLAUDE.md` 共 85 行，结构如下：

| 章节 | 行数 | 每次都需要？ | 建议 |
|---|---|---|---|
| 1. Think Before Coding | 8 | 是 | 保留，核心行为准则 |
| 2. Simplicity First | 8 | 是 | 保留 |
| 3. Surgical Changes | 11 | 是 | 保留 |
| 4. Goal-Driven Execution | 10 | 是 | 保留 |
| Precision Editing Protocol | 16 | **否** — 只在编辑时需要 | **移出为 skill** |
| Git Commit Convention | 4 | **否** — 已有 vcs-committer agent | **简化为一行指针** |

## 具体变更

### 1. Precision Editing Protocol → cc-skill

> **类型**：cc-skill（依 description 自动触发，不进 harness registry，不需要 `/dispatch`）

创建 `~/.claude/skills/precision-editing/SKILL.md`：

```yaml
---
name: precision-editing
description: Enforces surgical editing protocol with 100-line rule, locate-window-verify read pattern, and mega-edit prevention. Use when editing existing code files.
---

# Precision Editing Protocol

## Read: Locate-Window-Verify
- Grep target first → Read with offset/limit (max 300 lines).
- Never read from line 1 unless full survey needed.
- Include ±20 lines around target before editing.

## Write: 100-Line Rule
- Max 100 lines per Edit/Write.
- Larger changes → Edit-Verify cycle:
  1. Sub-change (≤100 lines).
  2. Syntax/build check.
  3. Repeat.
- 1000+ line renames → .patch or sed, not Edit.

## Forbidden
- No mega-edits: multiple fns in one Edit call.
- No blind overwrites: Grep/Read before writing.
```

### 2. Git Commit Convention → 保留原位（不下沉）

**不移出 CLAUDE.md。** 理由：vcs-committer 是可选路径，commit convention 是兜底约束；若用户绕过 `/dispatch` 直接 `git commit`，Claude 仍需知道格式规则和红线。两者拆开会失防。

当前 4 行原样保留，仅补一行指针：

```markdown
## Git Commit Convention
- gitmoji format: `gitmoji type(scope): desc`
- gitmoji mapping: feat/new→✨ fix→🐛 docs→📝 style→🎨 refactor→♻️ perf→⚡ test→✅ build→📦 ci→👷 chore→🔧 remove→🔥 wip→🚧
- Never append `Co-Authored-By: Claude ...`
- 复杂多文件提交 → `/dispatch vcs-committer`
```

### 3. 瘦身后的 CLAUDE.md 结构（约 70 行）

```markdown
# User-Level CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes.

## 1. Think Before Coding
[保留不变]

## 2. Simplicity First
[保留不变]

## 3. Surgical Changes
[保留不变]

## 4. Goal-Driven Execution
[保留不变]

## Git Commit Convention
[保留完整 4 行，见上方 §2]
```

> 注：移出 Precision Editing Protocol（~17 行）后预计减少约 17 行。目标上限 200 行，当前 85 行远未触及。Git Commit Convention 保留原位不变。

## 渐进式披露层级

```
~/.claude/CLAUDE.md（约 70 行，每次加载）
  │
  ├── Auto Memory / MEMORY.md          ← 自动加载（经验、纠正、项目上下文）
  │
  ├── ~/.claude/skills/precision-editing/SKILL.md    ← cc-skill：编辑时自动触发
  ├── ~/.agents/skills/claudemd-evolution/SKILL.md   ← harness-skill：/dispatch 手动触发
  └── ...其他 cc-skill / harness-skill
  │
  └── 项目根/CLAUDE.md          ← 进入项目时自动加载
```

## 收益

- 每次会话减少约 20 行 token 消耗（Precision Editing Protocol + Git Commit Convention 精简）
- 编辑协议只在需要时加载，不浪费一般会话的注意力预算
- 新增规则更容易找到合适的放置位置（CLAUDE.md vs Auto Memory vs skill）
