# CLAUDE.md 自我进化方案 — 架构总览

## 核心理念

CLAUDE.md 不是静态文档，而是**随使用积累经验、随模型升级而精简**的活文档。

进化机制分两层：
- **被动积累**：Stop hook 自动捕获会话中的经验教训，写入 Auto Memory
- **主动修剪**：定期 review，移除过时约束，晋升通用经验

## 参考来源

- [How Claude Code works in large codebases](https://claude.com/blog/how-claude-code-works-in-large-codebases-best-practices-and-where-to-start) — CLAUDE.md 分层、hooks 自我改进、模型升级后配置 review
- [Harnessing Claude's intelligence](https://claude.com/blog/harnessing-claudes-intelligence) — "what can I stop doing?" 思维、渐进式披露、context 管理
- [Hooks guide](https://code.claude.com/docs/en/hooks-guide) — Stop hook、SessionStart hook
- [Skill best practices](https://code.claude.com/docs/en/agents-and-tools/agent-skills/best-practices) — 渐进式披露模式、feedback loop

## 架构图

```
~/.claude/
├── CLAUDE.md                              # 用户级：每次会话加载的全局规则（< 200 行）
├── settings.json                          # hooks 配置
├── skills/
│   ├── claudemd-evolution/               # CLAUDE.md 进化 skill（手动触发）
│   │   └── SKILL.md
│   └── precision-editing/                # 精准编辑协议 skill
│       └── SKILL.md
└── projects/
    └── <project-slug>/
        └── memory/                        # Auto Memory（每次会话自动加载）
            ├── MEMORY.md                  # 索引（≤200 行）
            ├── feedback_*.md              # 行为反馈、纠正记录
            ├── project_*.md               # 项目上下文、模型漂移记录
            └── reference_*.md             # 外部系统位置
```

## 数据流

```
会话开始
  ↓
SessionStart hook → 提醒模型升级 review（阶段 5，matcher: startup）
  ↓
正常工作
  ↓
用户纠正 Claude → 告知 Claude 写入 Auto Memory（feedback 类型）（阶段 2）
  ↓
会话结束
  ↓
Stop prompt hook → 反思 → systemMessage 注入会话末尾 → [用户确认] → Auto Memory（阶段 1）
  ↓
用户执行 /dispatch claudemd-evolution
  ↓
Evolution Skill → 读取 Auto Memory → 分类规则 → 更新 CLAUDE.md（阶段 4）
  ↓
CLAUDE.md 瘦身：移出的规则 → Auto Memory 或 skills/（阶段 3）
```

## 不变量

| 约束 | 说明 |
|---|---|
| CLAUDE.md < 200 行 | 超出说明该拆分了 |
| Auto Memory MEMORY.md ≤ 200 行 | 系统自动管理，超出时截断 |
| Stop hook 只建议，不自动修改 | 写入 memory 需用户显式指示 |
| 先观察再写入 | 至少出现 2 次的纠正才晋升为候选；≥ 3 次才晋升为规则 |

## 文档索引

| 文件 | 内容 |
|---|---|
| [01-stop-hook.md](01-stop-hook.md) | Stop hook：被动捕获经验 |
| [02-memory-folder.md](02-memory-folder.md) | Auto Memory：经验持久化 |
| [03-progressive-disclosure.md](03-progressive-disclosure.md) | 渐进式披露：CLAUDE.md 瘦身 |
| [04-evolution-skill.md](04-evolution-skill.md) | Evolution Skill：定期 review |
| [05-model-upgrade-trigger.md](05-model-upgrade-trigger.md) | 模型升级触发器 |
| [06-roadmap.md](06-roadmap.md) | 实施路线图 |
