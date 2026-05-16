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

## 术语

| 术语 | 说明 |
|---|---|
| **cc-skill** | 放 `~/.claude/skills/<name>/SKILL.md`，依 description 由 CC 自动触发（无需 `/dispatch`） |
| **harness-skill** | 放 `~/.agents/skills/<name>/`（真身），在 `registry.yaml` 注册，软链至 `custom-harness/skills/`，通过 `/dispatch <name>` 手动调用 |
| **agent** | 与 harness-skill 同位，区别在语义上侧重"代理一次性任务"，harness-skill 侧重"持久协议" |
| **evolution skill** | 本方案的 claudemd-evolution 属于 **harness-skill**（手动 `/dispatch`，不自动触发） |
| **precision-editing** | 本方案的精准编辑协议属于 **cc-skill**（编辑时自动触发） |

## 架构图

```
~/.agents/skills/                          # harness-skill 真身（git 跟踪于 mycc 仓）
└── claudemd-evolution/                    # CLAUDE.md 进化 skill（手动 /dispatch 触发）
    └── SKILL.md

~/.claude/
├── CLAUDE.md                              # 用户级：每次会话加载的全局规则（< 200 行）
├── settings.json                          # hooks 配置
├── skills/
│   └── precision-editing/                 # cc-skill：description 自动触发（编辑类任务）
│       └── SKILL.md
├── custom-harness/
│   ├── registry.yaml                      # harness-skill 注册表（/dispatch 查表用）
│   └── skills/
│       └── claudemd-evolution             # → ~/.agents/skills/claudemd-evolution（软链）
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
Stop prompt hook → 反思 → systemMessage 建议（会话关闭前显示）→ [用户确认] → Auto Memory（阶段 1）
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
| 候选阈值 | 同类 feedback ≥ 2 次 → 标记为候选（02 和 04 中不再复述） |
| 晋升阈值 | 同类 feedback ≥ 3 次且跨项目通用 → 晋升至用户级 CLAUDE.md；项目特有保留为 project memory |

## 文档索引

| 文件 | 内容 |
|---|---|
| [01-stop-hook.md](01-stop-hook.md) | Stop hook：被动捕获经验 |
| [02-memory-folder.md](02-memory-folder.md) | Auto Memory：经验持久化 |
| [03-progressive-disclosure.md](03-progressive-disclosure.md) | 渐进式披露：CLAUDE.md 瘦身 |
| [04-evolution-skill.md](04-evolution-skill.md) | Evolution Skill：定期 review |
| [05-model-upgrade-trigger.md](05-model-upgrade-trigger.md) | 模型升级触发器 |
| [06-roadmap.md](06-roadmap.md) | 实施路线图 |
