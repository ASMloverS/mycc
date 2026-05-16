---
id: T14
title: settings.local.json 不被破坏
---

## 前置条件

T03 完成（`--apply` 已写入 `settings.json`）。

## 验证

```bash
git diff .claude/settings.local.json
```

期望：无任何 diff 输出（文件未被改动）。

```bash
python -c "
import json
d = json.load(open('.claude/settings.local.json', encoding='utf-8'))
allow = d.get('permissions', {}).get('allow', [])
print(f'settings.local.json allow 条目数：{len(allow)}')
# 期望：>=100（现有 ad-hoc allow 全部保留）
assert len(allow) >= 100, f'条目数 {len(allow)} < 100，疑似被清空'
print('OK')
"
```

两步均通过即通过。
