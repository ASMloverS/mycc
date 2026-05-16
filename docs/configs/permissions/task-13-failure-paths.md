---
id: T13
title: 脚本失败路径处理
---

## 前置条件

T02 完成。

## 验证

```bash
# 用例 1：损坏 JSON
python -c "
import pathlib
pathlib.Path('.claude/settings.json').write_text('{permissions: [}', encoding='utf-8')
"
python tools/perms-update.py
echo "exit=$?"
# 期望：退码 1，中文报错含行号信息，不写盘

# 用例 2：permissions 类型错误
python -c "
import json, pathlib
pathlib.Path('.claude/settings.json').write_text(
    json.dumps({'permissions': 'x'}), encoding='utf-8'
)
"
python tools/perms-update.py
echo "exit=$?"
# 期望：退码 1，报错「permissions 字段非对象类型」

# 用例 3：--target 白名单校验
python tools/perms-update.py --target invalid.json
echo "exit=$?"
# 期望：退码 1，报错说明合法 target 列表

# 清理
python -c "import pathlib; pathlib.Path('.claude/settings.json').unlink(missing_ok=True)"
```

三个用例均符合期望即通过。
