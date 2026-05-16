---
id: T01
title: 创建 tools/perms-update.recipe.json
---

## 目标

新建 `tools/perms-update.recipe.json`，包含 `allow` 和 `deny` 两个顶层数组，内容完整对应设计准则 Part A.1 + A.2。

## 实施

- 新建文件 `tools/perms-update.recipe.json`
- 根结构：`{ "allow": [...], "deny": [...] }`
- `deny` 数组：Part A.1 全部 87 条，保持分组顺序
- `allow` 数组：Part A.2 全部 43 条，保持分组顺序
- 编码 UTF-8，`json.dumps(indent=2, ensure_ascii=False)`，末尾换行

## 验证

```bash
python -c "
import json, sys
d = json.load(open('tools/perms-update.recipe.json', encoding='utf-8'))
assert isinstance(d.get('allow'), list), 'allow 非列表'
assert isinstance(d.get('deny'),  list), 'deny 非列表'
print(f'allow={len(d[\"allow\"])} deny={len(d[\"deny\"])}  OK')
"
```

期望输出：`allow=43 deny=87  OK`（条目数以实际写入为准，关键是两键均为列表且非空）。
