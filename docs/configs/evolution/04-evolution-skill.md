# 阶段 4：Evolution Skill — 定期 Review

## 目标

机械化 CLAUDE.md 的 review 流程。手动触发（`/dispatch claudemd-evolution`），读取 Auto Memory 中的证据，对每条规则进行分类判断，提出具体变更方案。

## Skill 定义

> **类型**：harness-skill（手动 `/dispatch claudemd-evolution` 触发，不自动加载）

**真身位置**：`~/.agents/skills/claudemd-evolution/SKILL.md`
**软链**：`~/.claude/custom-harness/skills/claudemd-evolution` → `~/.agents/skills/claudemd-evolution`

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

首先通过 Bash 推导 project-slug（CC 内部规则：去驱动器号 + 分隔符替换为 `-`）：

```bash
PROJECT_DIR="$(pwd)"
PROJECT_SLUG="$(echo "$PROJECT_DIR" | sed -e 's|^[A-Za-z]:||' -e 's|[/\\]|-|g' -e 's|^-||')"
# 例：C:/Workspace/Repositories/mycc → C--Workspace-Repositories-mycc（去掉驱动器后首字符是 -，再去掉）
MEMORY_DIR="$HOME/.claude/projects/$PROJECT_SLUG/memory"
```

若 `$MEMORY_DIR` 不存在，输出 `"no Auto Memory found for this project"` 并终止。

Read these files in order:
1. `$MEMORY_DIR/MEMORY.md` (Auto Memory index)
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

> 注：候选阈值与晋升阈值定义见 [00-overview.md §不变量](00-overview.md)，本文件不再复述。

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

## 注册到 dispatch

创建 SKILL.md 后，必须注册到 `registry.yaml` 才能通过 `/dispatch` 调用：

```bash
# 1. 创建真身目录
mkdir -p ~/.agents/skills/claudemd-evolution

# 2. 建软链（供 harness 查表）
ln -s ~/.agents/skills/claudemd-evolution \
      ~/.claude/custom-harness/skills/claudemd-evolution

# 3. 编辑 registry.yaml，在 skills: 段追加：
#    claudemd-evolution:
#      path: skills/claudemd-evolution/SKILL.md
#      desc: "Evolve CLAUDE.md from Auto Memory evidence"

# 4. 同步到 git 跟踪仓库
cp ~/.claude/custom-harness/registry.yaml \
   C:/Workspace/Repositories/mycc/custom-harness/claude/registry.yaml
# （提交走 vsc-committer）

# 5. 验证注册成功
/dispatch --help claudemd-evolution
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
