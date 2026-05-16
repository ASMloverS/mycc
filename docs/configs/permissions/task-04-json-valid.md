---
id: T04
title: 输出 JSON 合法性校验
---

## 前置条件

T03 完成（`.claude/settings.json` 已生成）。

## 验证

```bash
python -c "import json,sys; json.load(open(sys.argv[1], encoding='utf-8'))" .claude/settings.json
echo "exit=$?"
```

期望：无异常输出，`exit=0`。
