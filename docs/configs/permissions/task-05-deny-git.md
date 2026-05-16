---
id: T05
title: deny 实测 — git 危险操作
---

## 前置条件

T03 完成（`.claude/settings.json` 已生效），在 mycc 仓内新开 Claude Code session。

## 验证

在 Claude Code session 中依次要求执行以下命令，每条均应被 **deny 拒绝**：

| 命令 | 命中规则 |
|------|---------|
| `git push --force origin main` | `Bash(git push --force*)` |
| `git push origin main --force` | `Bash(git push * * --force*)` |
| `git push origin main -f` | `Bash(git push * * -f *)` |
| `git push --tags --force` | `Bash(git push --tags --force*)` |
| `git push --mirror` | `Bash(git push --mirror*)` |
| `git push --delete origin v1.0` | `Bash(git push --delete *)` |
| `git push origin :v1.0` | `Bash(git push * :*)` |
| `git push origin +main:main` | `Bash(git push * +*)` |
| `git reset --hard HEAD~1` | `Bash(git reset --hard*)` |
| `git clean -fd` | `Bash(git clean -fd*)` |
| `git clean -fdx` | `Bash(git clean -fdx*)` |
| `git branch -D feature` | `Bash(git branch -D *)` |
| `git filter-branch --tree-filter 'rm -f .env' HEAD` | `Bash(git filter-branch *)` |
| `git filter-repo --path src` | `Bash(git filter-repo *)` |
| `git update-ref -d refs/heads/old` | `Bash(git update-ref -d *)` |
| `git tag -d v1.0` | `Bash(git tag -d *)` |
| `git remote rm origin` | `Bash(git remote rm *)` |
| `git remote remove upstream` | `Bash(git remote remove *)` |
| `git worktree remove --force /tmp/wt` | `Bash(git worktree remove --force*)` |
| `git checkout -- src/main.py` | `Bash(git checkout -- *)` |
| `git checkout .` | `Bash(git checkout .)` |
| `git restore --staged --worktree src/` | `Bash(git restore --staged --worktree*)` |

全部被 deny 即通过。
