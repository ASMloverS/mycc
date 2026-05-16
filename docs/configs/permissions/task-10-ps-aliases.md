---
id: T10
title: deny 实测 — PowerShell 别名 normalize 验证
---

## 前置条件

T03 完成，Windows 11 环境；分别在 **PowerShell 5**（`powershell.exe`）和 **PowerShell 7**（`pwsh.exe`）各跑一次。

## 验证

在 Claude Code session 中，切换至 PowerShell 上下文，依次要求执行以下命令，每条均应被 **deny 拒绝**：

| 命令（别名形式） | 期望命中规则 | 备注 |
|------|---------|------|
| `iex "ls"` | `PowerShell(Invoke-Expression *)` | `iex` 应 normalize 为 `Invoke-Expression` |
| `rm -Recurse C:\tmp` | `PowerShell(Remove-Item * -Recurse*)` | `rm` → `Remove-Item` |
| `ri -Force C:\tmp\x` | `PowerShell(ri * -Force*)` | `ri` 直名条目 |
| `rmdir -Recurse C:\tmp` | `PowerShell(rmdir * -Recurse*)` | `rmdir` → `Remove-Item` |

## 处置规则

若任一项**不被 deny**，说明官方 normalize 不覆盖该别名（关键规则 §7 假设失效），需：

1. 将该别名别名条目从 `deny` 移除（避免误导）
2. 用 `PreToolUse` hook 兜底拦截
3. 在 `tools/perms-update.recipe.json` 对应条目添加注释说明

PS5 和 PS7 均通过，才算全通过。
