---
id: T12
title: 跨平台验证 — Debian 12 (WSL)
---

## 前置条件

T01–T04 在 Windows 11 已通过。Debian 12 WSL 环境可用，Python 3.x 已安装。

## 验证

在 WSL Debian 12 shell 中，切换至 mycc 仓目录后，重跑以下步骤（脚本零改动）：

```bash
# T01 等价验证：recipe JSON 可正常解析
python3 -c "
import json
d = json.load(open('tools/perms-update.recipe.json', encoding='utf-8'))
assert isinstance(d.get('allow'), list)
assert isinstance(d.get('deny'), list)
print('recipe OK')
"

# T03 等价验证：四步冒烟
python3 tools/perms-update.py --help
python3 -c "import pathlib; pathlib.Path('.claude/settings.json').unlink(missing_ok=True)"
python3 tools/perms-update.py           # 期望：+N allow, +M deny，不创建文件
python3 tools/perms-update.py --apply   # 期望：创建 settings.json，无 .bak
python3 tools/perms-update.py --apply   # 期望：+0 allow, +0 deny，有 .bak

# T04 等价验证：JSON 合法性
python3 -c "import json,sys; json.load(open(sys.argv[1]))" .claude/settings.json
echo "exit=$?"
```

并在 Debian 12 WSL 的 Claude Code session 中，抽选 T05–T09 中各 2 条 Linux deny 用例验证（Windows 特有命令 `del /s`、`rd /s` 等跳过）。

全部通过即通过。
