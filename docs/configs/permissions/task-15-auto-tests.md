---
id: T15
title: 自动化回归测试套件（可选）
---

## 目标

新建 `tools/test_perms_update.py`，参照 `tools/test_sync_config.py` 风格，用 pytest 自动化覆盖 T03–T14 核心场景。

## 覆盖范围

| 测试 | 对应任务 |
|------|---------|
| deny 命中：T05–T09 全部命令的规则字符串出现在 deny 列表中 | T05–T09 |
| allow 不误拦：T11 全部命令的规则字符串出现在 allow 列表且不在 deny 中 | T11 |
| 幂等性：连跑 2 次 `--apply`，第二次 `new_allow=0, new_deny=0` | T03 |
| JSON 合法性 round-trip：写入再解析无异常 | T04 |
| 跨语法重复 warning：recipe 中同名工具冒号/空格语法共存时触发 WARN | T02 |
| 失败路径：损坏 JSON / 类型错误 / --target 非法各退码 1 | T13 |

## 实施

- 测试文件约 120 行
- 用 `tmp_path` fixture 隔离文件系统，不改动项目 `.claude/` 目录
- 不依赖外部 Claude Code session（纯 Python 单元 + 集成测试）

## 验证

```bash
pytest tools/test_perms_update.py -v
```

期望：全部 passed，无 failed/error。
