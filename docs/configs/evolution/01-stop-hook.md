# 阶段 1：Stop Hook — 被动捕获经验

## 目标

每次会话结束时，利用 `Stop` prompt hook 让 Claude 反思本次会话，自动识别 CLAUDE.md 中缺失、过时或可改进的规则。

## 设计决策

**为什么用 prompt hook 而不是 command hook？**
- prompt hook 使用 LLM 判断"是否需要改进"，需要语义理解
- command hook 适合确定性规则（格式化、拦截），不适合反思

**为什么只建议不自动修改？**
- 避免误报：一次性的操作失误不应成为永久规则
- 需要至少出现 2 次的模式才有资格晋升为规则（见 [02-memory-folder.md](02-memory-folder.md)）

## 配置

在 `~/.claude/settings.json` 的 `hooks` 中添加：

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "prompt",
            "prompt": "Review this session. Check:\n1. Did you repeat a mistake that CLAUDE.md should have prevented?\n2. Did you need instructions NOT found in CLAUDE.md?\n3. Did any CLAUDE.md rule feel outdated or counterproductive with your current capabilities?\n\nIf any apply, respond with ONLY valid JSON, no other text:\n{\"systemMessage\": \"CONCISE suggestion: what to ADD/REMOVE/MODIFY in CLAUDE.md. Be specific.\"}\n\nIf nothing to improve, respond with an empty object:\n{}"
          }
        ]
      }
    ]
  }
}
```

## 行为

| hook 返回 | 效果 |
|---|---|
| `{}` | 会话正常结束，无动作 |
| `{"systemMessage": "..."}` | Stop hook 在会话真正关闭前触发；suggestion 在会话末尾显示给用户 |
| 非 JSON 或解析失败 | 静默跳过，不阻塞会话结束 |

注意：`systemMessage` 是 CC hooks 协议中 Stop 事件支持的注入字段。用户看到建议后，需手动指示 Claude 将其写入 Auto Memory（`feedback` 类型）。首次配置后建议用 `claude --debug` 验证 hook 是否正常触发（参见 [06-roadmap.md §阶段 1 验证](06-roadmap.md)）。

## 工作流

1. Claude 完成任务，触发 Stop 事件
2. Prompt hook 将会话摘要发送给 Haiku（默认模型）评估
3. 如果有改进建议：
   - 建议通过 `systemMessage` 出现在会话末尾
   - 用户看到后可指示 Claude 将其作为 `feedback` 类型记录到 Auto Memory
   - 或者忽略（一次性问题不值得记录）
4. 如果是重复出现的模式（Auto Memory 中已有类似 feedback），下次 `/dispatch claudemd-evolution` 时会考虑晋升为正式规则

## 注意事项

- Prompt hook 默认使用 Haiku 模型，成本低、速度快
- Hook 超时 30 秒，对简单会话足够
- 建议应该是具体的，例如："添加规则：本项目使用 `cmake --build build_ninja` 而非 `cmake --build build`"，而非泛泛的 "改进构建说明"
- Stop hook 仅输出建议文本，写入 memory 需用户显式指示。hooks 作为独立子进程运行，不受 plan mode 或其他会话模式约束
