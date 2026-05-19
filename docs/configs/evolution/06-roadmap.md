# 实施路线图

## 优先级排序

| 阶段 | 文档 | 优先级 | 工作量 | 前置依赖 | 收益 |
|---|---|---|---|---|---|
| 1. Stop Hook | [01-stop-hook.md](01-stop-hook.md) | **P0** | 低（加一段 JSON） | 无 | 最快见效，零侵入，自动捕获经验 |
| 2. Auto Memory | [02-memory-folder.md](02-memory-folder.md) | **P0** | 零（内置功能） | 无 | 经验不再流失，Claude Code 已内置 |
| 3. 瘦身 | [03-progressive-disclosure.md](03-progressive-disclosure.md) | **P1** | 中（创建 skill + 修改 CLAUDE.md） | 无 | 减少每次会话 token，立即可做 |
| 4. Evolution Skill | [04-evolution-skill.md](04-evolution-skill.md) | **P1** | 中（写 SKILL.md） | 阶段 2 | 机械化 review 流程 |
| 5. 模型升级触发 | [05-model-upgrade-trigger.md](05-model-upgrade-trigger.md) | **P2** | 低（加一段 JSON） | 阶段 4 | 防止配置腐烂 |

## 实施顺序

```
Week 1: 阶段 1（Stop Hook）
  └── 在 settings.json 添加 Stop prompt hook

Week 2: 阶段 3
  ├── 创建 ~/.claude/skills/precision-editing/SKILL.md
  ├── 修改 CLAUDE.md（移出 Precision Editing Protocol）
  └── 验证：正常编辑任务仍然遵循 precision editing 规则

Week 3: 阶段 4
  ├── 创建 ~/.agents/skills/claudemd-evolution/SKILL.md
  ├── 建软链：~/.claude/custom-harness/skills/claudemd-evolution
  ├── 注册到 registry.yaml + git 同步
  └── 试运行：/dispatch claudemd-evolution

Week 4: 阶段 5
  ├── 在 settings.json 添加 SessionStart hook（含 matcher: "startup"）
  └── 观察：首次新会话是否正确触发提醒（resume 不触发）
```

> 阶段 2（Auto Memory）零工作量：Claude Code v2.1.59+ 已内置，直接使用即可。

## 验证清单

### 阶段 1 验证
- [ ] `claude --debug` 启动，结束会话后在 debug 输出中确认 Stop hook 触发记录
- [ ] hook 返回 `{"systemMessage": "..."}` 时，建议在会话末尾可见
- [ ] 建议内容具体且可操作（非泛泛而谈）
- [ ] 用 `jq` 验证 hook 输出合法：`echo '{"systemMessage":"test"}' | jq .`（期望无报错）

### 阶段 2 验证
- [ ] `~/.claude/projects/<slug>/memory/MEMORY.md` 存在
- [ ] Claude 能通过 Auto Memory 跨会话记住 feedback
- [ ] `/memory` 命令正常显示当前项目记忆

### 阶段 3 验证
- [ ] CLAUDE.md 行数减少约 20 行（移出 Precision Editing Protocol + 精简 Git Commit Convention）
- [ ] `~/.claude/skills/precision-editing/SKILL.md` 存在且在编辑任务中触发
- [ ] Git commit 委托给 vcs-committer 正常工作

### 阶段 4 验证
- [ ] `/dispatch --help claudemd-evolution` 显示 desc 字段（确认 registry 注册成功）
- [ ] `/dispatch claudemd-evolution` 在空 Auto Memory 项目下输出 `"no Auto Memory found"` 并退出
- [ ] 在有 Auto Memory 的项目下输出结构化提案（REMOVE/SIMPLIFY/RELOCATE/ADD）
- [ ] 用户确认后才修改 CLAUDE.md

### 阶段 5 验证
- [ ] 启动新会话时，SessionStart hook additionalContext 注入成功
- [ ] resume/compact 场景不触发（matcher: "startup" 生效）

## Windows 兼容性

本项目的 hooks 配置已经考虑 Windows 环境：
- command hook 显式声明 `"shell": "bash"` 确保在 Git Bash 环境中运行
- 文件路径使用 `~/.claude/` 在 Claude Code 的 bash 环境中正常解析
- JSON 输出（`hookSpecificOutput.additionalContext`）通过 `echo '...'` 输出，注意 Windows bash 中使用单引号包裹

**故障排查**：

| 症状 | 排查步骤 |
|---|---|
| Stop hook 无任何输出 | `claude --debug` 启动，检查 debug 日志中 `hooks` 相关条目 |
| JSON 解析失败 / 静默跳过 | 手动运行 hook command，确认 stdout 为合法 JSON（`echo '...' | jq .`） |
| `echo '...'` 单引号失效（Windows）| 改用 `printf '%s' '...'`，或切换为 PowerShell hook（`"shell": "powershell"`，JSON 用双引号 + 转义） |
| hook 有输出但 systemMessage 不显示 | 确认 CC 版本 ≥ 2.1.59；用 `claude --debug` 确认 Stop 事件是否触发 prompt hook |

## 合并后的 settings.json hooks 配置

阶段 1 和阶段 5 分别向 `settings.json` 添加 hooks，合并后完整配置：

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
    ],
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

## 回滚方案

| 阶段 | 回滚方式 |
|---|---|
| 阶段 1 (Stop Hook) | 从 `settings.json` 移除 `hooks.Stop` 段 |
| 阶段 2 (Auto Memory) | 无需回滚（内置功能，不影响其他配置） |
| 阶段 3 (瘦身) | 将 precision-editing 内容还原到 `CLAUDE.md`，删除 skill 目录 |
| 阶段 4 (Evolution Skill) | 删除 `~/.agents/skills/claudemd-evolution/`，移除软链，从 registry.yaml 删除注册项 |
| 阶段 5 (SessionStart Hook) | 从 `settings.json` 移除 `hooks.SessionStart` 段 |

## 潜在风险

| 风险 | 缓解 |
|---|---|
| Stop hook 误报（一次性问题被建议为规则） | 晋升需要同类 feedback ≥ 3 次 |
| Auto Memory MEMORY.md 超 200 行被截断 | 及时清理已晋升的 feedback 条目 |
| 过度瘦身导致 CLAUDE.md 失去关键约束 | Evolution skill 输出 KEEP 列表供确认 |
| Hook 超时影响体验 | Prompt hook 30s 超时；command hook 10min 超时 |
| Windows bash 中 JSON 单引号转义 | 测试 `echo '{"key": "val"}'` 输出是否完整 |
