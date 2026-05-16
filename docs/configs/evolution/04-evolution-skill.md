# 阶段 4：Evolution Skill — 定期 Review

## 目标

机械化 CLAUDE.md 的 review 流程。手动触发（`/dispatch claudemd-evolution`），读取 Auto Memory 中的证据，对每条规则进行分类判断，提出具体变更方案。

## Skill 定义

位置：`~/.claude/skills/claudemd-evolution/SKILL.md`

```yaml
---
name: claudemd-evolution
description: Reviews and evolves CLAUDE.md files. Reads Auto Memory logs,
  classifies each rule as needed/stale/relocatable, proposes additions
  from recurring corrections. Trigger via /dispatch or after major model
  releases.
disable-model-invocation: true
---

# CLAUDE.md Evolution

You are a CLAUDE.md maintenance agent. Your job is to review and evolve
the user's CLAUDE.md based on accumulated evidence.

## Step 1: Gather evidence

Read these files in order:
1. `~/.claude/projects/<current-project-slug>/memory/MEMORY.md` (Auto Memory index)
2. Individual feedback_*.md and project_*.md files referenced in MEMORY.md
3. `~/.claude/CLAUDE.md` (current rules)

## Step 2: Classify each CLAUDE.md rule

For every rule/section in CLAUDE.md, evaluate:

| Category | Criteria | Action |
|---|---|---|
| **KEEP** | Model still needs it; prevents real mistakes | No change |
| **REMOVE** | Model no longer makes this mistake; rule is stale | Delete |
| **SIMPLIFY** | Rule is verbose; can be expressed in fewer words | Shorten |
| **RELOCATE** | Not needed every session; belongs in skill/memory | Move out |

Classification heuristics:
- If Auto Memory has NO feedback entries matching this rule in 90 days → candidate REMOVE
- If project memory notes the model improved in this area → candidate REMOVE
- If the rule is only relevant to specific tasks → candidate RELOCATE to skill
- If feedback memory has a pattern with ≥ 3 entries and no matching rule → candidate ADD

> 注：同类 feedback 出现 ≥ 2 次标记为"候选"，≥ 3 次正式提议晋升。两级过滤一次性噪声。

## Step 3: Propose additions from memory

For each feedback and project memory entry:
- Group by pattern (keyword cluster / correction target, not exact text)
- If pattern count ≥ 3 AND no existing rule covers it → propose ADD
- If project-specific → propose adding to project-level CLAUDE.md instead

## Step 4: Present changes

Output a structured proposal:

```
## CLAUDE.md Evolution Proposal — [date]

### Rules to REMOVE
- [rule]: [reason] (evidence: [Auto Memory entry])

### Rules to SIMPLIFY
- [rule]: [before] → [after] (reason: ...)

### Rules to RELOCATE
- [rule] → [destination] (reason: ...)

### Rules to ADD
- [new rule]: (source: [memory entry]) [proposed text]

### No-action rules (confirmed KEEP)
- [list of rules that are still valid]

### Memory cleanup suggestions
- [feedback entries that can be removed after promotion]
```

## Step 5: Apply (only after user approval)

After user confirms:
1. Update CLAUDE.md
2. Ask user to clean up promoted feedback memory entries via `/memory`
3. Record the change summary in Auto Memory (project type):

```markdown
---
name: project-evolution-log-YYYY-MM
description: CLAUDE.md evolution run on YYYY-MM-DD
metadata:
  type: project
---

## YYYY-MM-DD Evolution
- REMOVED: "[rule]" (stale after [model version])
- ADDED: "[rule]" (feedback count: 3)
- SIMPLIFIED: [section] → [summary]
```
```

## 使用方式

```bash
# 手动触发
/dispatch claudemd-evolution

# 建议频率
# - 每月一次
# - 大模型发布后一周内
# - 感觉 Claude 表现下降时
```

## 与其他阶段的关系

```
阶段 1 (Stop Hook)       → 捕获经验 → systemMessage → 用户确认 → Auto Memory
阶段 2 (Auto Memory)     → 存储经验 → 等待晋升
阶段 4 (本阶段, Skill)    → 读取 Auto Memory → 评审 → 更新 CLAUDE.md
阶段 3 (Progressive)      → 作为评审的副产品执行
阶段 5 (Model Upgrade)    → 提醒执行本 Skill
```
