---
id: T03
title: 脚本 CLI 四步冒烟测试
---

## 前置条件

T01、T02 完成。

## 验证（按序执行）

```bash
# 步骤 1：help
python tools/perms-update.py --help
# 期望：打印用法，退码 0

# 步骤 2：dry-run（原文件不存在时）
python -c "
import pathlib; p = pathlib.Path('.claude/settings.json')
p.unlink(missing_ok=True)
"
python tools/perms-update.py
# 期望：输出 "+N allow, +M deny"，.claude/settings.json 不存在，退码 0

# 步骤 3：apply（首次写入）
python tools/perms-update.py --apply
# 期望：创建 .claude/settings.json，无 .bak 文件，退码 0

# 步骤 4：idempotency（二次 apply）
python tools/perms-update.py --apply
# 期望：输出 "+0 allow, +0 deny"，生成 .claude/settings.json.bak，退码 0
```

四步全部满足期望即通过。
