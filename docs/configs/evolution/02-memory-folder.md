# 阶段 2：Auto Memory — 经验持久化

## 目标

利用 Claude Code 内置的 Auto Memory 系统持久化经验。CLAUDE.md 自身保持精简（< 200 行），具体经验通过 Auto Memory 跨会话积累，无需自定义 memory 目录。

## 官方 Auto Memory 机制

Claude Code v2.1.59+ 内置 Auto Memory，自动维护：

```
~/.claude/projects/<project-slug>/memory/
├── MEMORY.md          # 索引文件（每次会话自动加载，≤200 行 / 25KB）
├── feedback_xxx.md    # 反馈类记忆：行为纠正、偏好指导
├── user_xxx.md        # 用户类记忆：角色、背景、知识水平
├── project_xxx.md     # 项目类记忆：目标、决策、上下文
└── reference_xxx.md   # 引用类记忆：外部系统、资源位置
```

- MEMORY.md 作为索引，每条指向具体记忆文件（每条 ≤150 字符）
- 超过 200 行后截断；具体内容存放在分类文件中
- `/memory` 命令可查看和管理当前项目的记忆

## Memory 类型与经验映射

原 custom-harness/memory/ 的概念映射到官方 memory types：

| 原概念 | 官方类型 | 说明 |
|---|---|---|
| `learnings.md`（经验教训） | `feedback` | 行为偏好、工作方式指导 |
| `corrections.md`（被纠正的错误） | `feedback` | "不要做 X，因为 Y" 类型的纠正 |
| `model-drift-notes.md`（模型行为变化） | `project` | 当前模型版本的特定约束或能力 |
| `evolution-log.md`（review 记录） | `project` | evolution skill 执行历史 |
| 项目特有约定 | `project` | 依赖、构建命令、架构决策 |
| 外部系统位置 | `reference` | 文档 URL、工具路径、issue tracker |

## feedback 类型记忆格式

纠正类记忆（对应原 corrections.md）：

```markdown
---
name: feedback-build-command
description: Use cmake --build build_ninja not build in this project
metadata:
  type: feedback
---

不使用 `cmake --build build`，改用 `cmake --build build_ninja`。

**Why:** 项目专用 Ninja 构建目录。
**How to apply:** 每次 cmake build 操作时。
```

经验教训类（对应原 learnings.md）：

```markdown
---
name: feedback-pytest-invocation
description: Run pytest as python -m pytest on Windows
metadata:
  type: feedback
---

Windows 环境下使用 `python -m pytest` 而非 `pytest`（路径问题）。

**Why:** Windows PATH 不总是将 pytest 加入可执行路径。
**How to apply:** 所有 Python 测试执行命令。
```

## 晋升机制

经验积累路径（通过 Auto Memory + Evolution Skill）：

```
会话结束 → Stop hook 建议
  ↓ 用户确认值得记录
Auto Memory（feedback/project type）
  ↓ 同类 feedback 出现 ≥ 2 次（候选）
  ↓ 出现 ≥ 3 次且非项目特有
/dispatch claudemd-evolution → 晋升为 CLAUDE.md 规则
```

**晋升标准**：
- 同类 feedback memory 积累 ≥ 2 条时，标记为"候选晋升"
- ≥ 3 条且确认跨项目通用 → 晋升至用户级 `~/.claude/CLAUDE.md`
- 项目特有 → 保留为 `project` 类型 memory，或写入项目级 CLAUDE.md
- 模型版本特定的行为（模型升级后可能消失）→ 记录为 `project` type，注明版本

## 使用方式

```bash
# 查看当前项目 memory
/memory

# 告知 Claude 保存一条 feedback
"请把这个纠正保存到 Auto Memory 作为 feedback 类型"

# Claude 自动写入 MEMORY.md + 对应分类文件
```

## 与 CLAUDE.md 的关系

| 内容类型 | 存放位置 | 加载时机 |
|---|---|---|
| 每次会话都需要的全局规则 | `~/.claude/CLAUDE.md` | 每次自动 |
| 跨会话积累的行为反馈 | Auto Memory（feedback） | 每次自动（≤200 行索引） |
| 当前项目的上下文 | Auto Memory（project） | 每次自动 |
| 项目特有的详细约定 | 项目根 `CLAUDE.md` | 进入项目时自动 |
| 专业领域知识 | `~/.claude/skills/*/SKILL.md` | 按需触发 |
