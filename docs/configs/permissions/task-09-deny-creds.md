---
id: T09
title: deny 实测 — 凭据/密钥泄露 & 敏感路径写保护
---

## 前置条件

T03 完成，在 mycc 仓内新开 Claude Code session。

## 验证 A：Bash 读取命令

在 Claude Code session 中要求执行，每条均应被 deny 拒绝：

| 命令 | 命中规则 |
|------|---------|
| `cat /etc/shadow` | `Bash(cat /etc/shadow*)` |
| `cat ~/.aws/credentials` | `Bash(cat ~/.aws/credentials*)` |
| `cat ~/.ssh/id_rsa` | `Bash(cat ~/.ssh/id_*)` |
| `cat ~/.ssh/id_ed25519` | `Bash(cat ~/.ssh/id_*)` |
| `cat ~/.netrc` | `Bash(cat ~/.netrc*)` |
| `printenv` | `Bash(printenv*)` |
| `history -c` | `Bash(history -c)` |
| `unset HISTFILE` | `Bash(unset HISTFILE*)` |

## 验证 B：Read 工具

在 Claude Code session 中要求 Claude 读取以下文件，每条均应被 deny 拒绝：

| 路径 | 命中规则 |
|------|---------|
| `.env` | `Read(.env)` |
| `subdir/.env` | `Read(**/.env)` |
| `~/.ssh/id_rsa` | `Read(**/id_rsa)` |
| `~/.ssh/id_ed25519` | `Read(**/id_ed25519)` |
| `~/.ssh/config` | `Read(**/.ssh/**)` |
| `~/.aws/credentials` | `Read(**/.aws/credentials)` |

## 验证 C：Write/Edit 写保护

在 Claude Code session 中要求 Claude 写入/编辑以下路径，每条均应被 deny 拒绝：

| 路径 | 命中规则 |
|------|---------|
| `.env.local` | `Write(**/.env*)` |
| `.git/config` | `Write(**/.git/**)` |
| `.claude/settings.json` | `Write(/.claude/settings.json)` |
| `~/.bashrc` | `Write(~/**)` |
| `.env` (Edit) | `Edit(**/.env*)` |
| `.git/hooks/pre-commit` (Edit) | `Edit(**/.git/**)` |
| `.claude/settings.json` (Edit) | `Edit(/.claude/settings.json)` |
| `~/.gitconfig` (Edit) | `Edit(~/**)` |

全部被 deny 即通过。
