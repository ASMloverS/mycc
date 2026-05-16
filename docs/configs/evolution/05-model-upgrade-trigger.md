# 阶段 5：模型升级触发器

## 目标

在新会话启动时，自动提醒执行 CLAUDE.md review。仅在真正的新会话触发，不干扰 resume/compact。

## 背景

来自 [Harnessing Claude's intelligence](https://claude.com/blog/harnessing-claudes-intelligence) 的核心洞察：

> Agent harnesses encode assumptions about what Claude can't do on its own, but those assumptions grow stale as Claude gets more capable.

具体案例：
- Sonnet 4.5 有"上下文焦虑"（接近 limit 时提前结束），Opus 4.5 不再有此问题 → 之前添加的 context reset 变成死代码
- 为旧模型写的单文件拆分规则阻碍了新模型的跨文件协调编辑能力

## 配置

在 `~/.claude/settings.json` 的 `hooks` 中添加 SessionStart hook：

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "echo '{\"hookSpecificOutput\": {\"additionalContext\": \"After a model upgrade, run /dispatch claudemd-evolution to review CLAUDE.md for stale rules. Check Auto Memory project entries for recorded model-drift notes.\"}}'",
            "shell": "bash"
          }
        ]
      }
    ]
  }
}
```

**关键配置说明：**
- `matcher: "startup"` — 限定只在新会话启动时触发，resume 和 compact 场景不触发
- `hookSpecificOutput.additionalContext` — command hook 将内容注入 Claude 上下文的官方字段；command hook 的 stdout 不会自动进入上下文
- 去除了 `${CLAUDE_MODEL}` 引用：该环境变量不在 Claude Code 的导出列表，值始终为空

## 工作流

```
用户启动 Claude Code（新会话）
  ↓
SessionStart hook 触发（matcher: startup，不含 resume/compact）
  ↓
additionalContext 注入 Claude 上下文
  ↓
用户决定是否执行 /dispatch claudemd-evolution
  ↓
Evolution skill 读取 Auto Memory 中的 project/feedback 记录
  ↓
对比当前模型能力与 CLAUDE.md 中的假设
  ↓
提出移除/简化建议
```

## model-drift-notes 维护

模型升级后的行为变化通过 Auto Memory（`project` 类型）记录：

```bash
# 告知 Claude 保存模型变化观察
"请把以下观察保存为 Auto Memory 的 project 类型：
Opus 4.7 跨文件重构更流畅，不再需要显式指示逐文件编辑"
```

对应 memory 文件格式示例：

```markdown
---
name: project-model-drift-opus47
description: Opus 4.7 cross-file refactor improved - precision editing rules may be overly strict
metadata:
  type: project
---

Opus 4.7 跨文件重构能力增强，precision-editing skill 的 100-line rule 可能过严。

**Why:** 新模型上下文协调更好，不需要强制拆分编辑。
**How to apply:** 下次 /dispatch claudemd-evolution 时评估是否放宽 100-line 限制。
```

## 替代方案

如果不想用 SessionStart hook，可在 `CLAUDE.md` 加一行：

```markdown
## Maintenance
- After every major model upgrade, run `/dispatch claudemd-evolution`
```

缺点：每次会话都加载这一行，浪费注意力预算。SessionStart hook 只在启动时触发一次，更精准。
