---
id: T02
title: 创建 tools/perms-update.py
---

## 目标

新建 `tools/perms-update.py`，实现设计准则 Part B 全部规格。

## 实施要点

- 头行 `#!/usr/bin/env python3`，仅用标准库（`argparse json sys pathlib`）
- 启动时从 `tools/perms-update.recipe.json` 加载推荐集；文件缺失/结构错误 → 退码 1 + CN 报错
- `find_repo_root()`：从 CWD 向上找含 `.claude/` 目录的祖先，找不到则用 CWD
- `--target` 白名单校验：仅允许 `settings.json` / `settings.local.json`，否则退码 1
- upsert 逻辑：精确字符串去重，现有条目在前，新增追加在后；不删任何现有条目
- 写盘前生成 `<target>.bak`（覆盖式，仅当原文件存在时）
- 跨语法重复 warning：同一工具下既有 `Tool(x:*)` 又新增 `Tool(x *)` 时打印 WARN（不阻断）
- 输出格式：`+N allow, +M deny  →  <target_path>`
- `--diff` 模式：逐行打印将新增条目；与 `--apply` 同时使用时先 diff 再写盘

## 验证

```bash
python tools/perms-update.py --help
```

期望：打印用法说明，退码 0。
