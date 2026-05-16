---
id: T11
title: allow 通畅性回归 — 常用命令不误拦
---

## 前置条件

T03 完成，在 mycc 仓内新开 Claude Code session。

## 验证

在 Claude Code session 中依次要求执行以下命令，每条均应**静默通过**（无 deny / ask 提示）：

| 命令 | allow 来源 |
|------|---------|
| `git status` | 内建只读白名单 + `Bash(git *)` |
| `git log --oneline -5` | 同上 |
| `git diff HEAD~1` | 同上 |
| `cargo build` | `Bash(cargo *)` |
| `cargo test` | `Bash(cargo *)` |
| `cargo fmt` | `Bash(cargo *)` |
| `python -m pytest` | `Bash(python *)` |
| `python3 -m mypy src/` | `Bash(python3 *)` |
| `pytest -q` | `Bash(pytest *)` |
| `cmake --build build` | `Bash(cmake *)` |
| `make -j4` | `Bash(make *)` |
| `npm test` | `Bash(npm *)` |
| `npm run build` | `Bash(npm *)` |
| `go build ./...` | `Bash(go *)` |
| `go test ./...` | `Bash(go *)` |
| `ruff check src/` | `Bash(ruff *)` |
| `eslint src/` | `Bash(eslint *)` |

**注**：`python -c` 已列入 deny，不在本回归用例中。

全部静默通过（无意外拦截）即通过。
