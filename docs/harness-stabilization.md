# dispatch / custom-harness 不稳节点脚本化

## 背景

`~/.claude/custom-harness/` 是自建的 dispatch 路由 + agent/skill 仓库。`dispatch.py` 已脚本化路由，但**编排型 agent**（`dev-cycle` / `bug-fixer` / `doc-fixer`）内部仍把若干**纯机械流程**交给 LLM 执行，导致执行不稳：

| 节点 | 位置 | 问题 |
|------|------|------|
| TASKS 状态更新 | `dev-cycle.md` Step 4 | LLM 自己 Glob/抽 ID/识别 marker/精改单行，易改错行/漏改 |
| Review 严重度解析 + 循环判定 | `dev-cycle.md` Step 3 / `bug-fixer.md` Step 5 | LLM 肉眼数 CRIT/MAJ 决定是否继续循环，reviewer 输出漂移即崩 |
| dispatch shape 判断 | `skills/dispatch/SKILL.md` Flow#3 | LLM 看 stdout 第一字符判 array vs object，脆弱 |
| commit 消息校验 | `vsc-committer.md` Step 2 | "must begin with emoji — regenerate if not"全靠 LLM 自检 |

四类节点均为纯机械逻辑，下沉到脚本后 LLM 只需"理解意图 / 生成内容"。

---

## 目标

1. `dev-cycle` Step 4 退化为单行 Bash 调脚本
2. `code-reviewer` 强制输出 `<REVIEW_RESULT>JSON</REVIEW_RESULT>`，循环控制由脚本解析
3. `dispatch.py` 始终输出 `{mode, payloads}` envelope，SKILL 不再判 shape
4. `vsc-commit.py` 内置 gitmoji 正则，不通过即拒收

---

## 文件变更清单

### 新增脚本（按 agent 分组）

```
~/.claude/custom-harness/bin/
├── dev-cycle/
│   └── task-status.py      ← 替代 dev-cycle Step 4 全部 LLM 启发式
└── review/
    └── parse-review.py     ← 解析 <REVIEW_RESULT> 块，判定 pass/fail
```

### 修改文件

| 文件 | 改动要点 |
|------|---------|
| `~/.claude/custom-harness/bin/dispatch.py` | 输出统一 envelope `{mode, payloads}` |
| `~/.claude/custom-harness/bin/vsc-commit.py` | 新增 `validate_msg()` + 集成到 commit 路径 |
| `~/.claude/custom-harness/agents/dev-cycle.md` | Step 3 接 parse-review.py；Step 4 调 task-status.py |
| `~/.claude/custom-harness/agents/code-reviewer.md` | 末尾追加 `<REVIEW_RESULT>` 输出契约 |
| `~/.claude/custom-harness/agents/code-implementer.md` | 末尾追加 `<IMPL_RESULT>` 输出契约 |
| `~/.claude/custom-harness/agents/bug-fixer.md` | Step 5 改用 parse-review.py |
| `~/.claude/custom-harness/agents/vsc-committer.md` | 删除 LLM 自检逻辑，改为"exit 5 则重新生成" |
| `~/.claude/skills/dispatch/SKILL.md` | Flow#3 改读 `envelope.mode` |

**兼容性策略：硬切**，旧用法（裸 array/object；LLM 自检 emoji）立即失效，不保留 fallback。

---

## 详细设计

### 1. `~/.claude/custom-harness/bin/dev-cycle/task-status.py`

```
Usage:
  python ~/.claude/custom-harness/bin/dev-cycle/task-status.py \
    --spec <spec.md> --to <done|in-progress|pending|cancelled>
    [--tasks-index <path>] [--dry-run]
```

**算法**：

1. **TASKS 索引定位**：优先 `--tasks-index`；否则从 spec 目录向上找 `TASKS.md` / `tasks.md`；找不到 → exit 2
2. **Task ID 抽取**（按优先级）：
   - spec frontmatter `task: T18`
   - spec 文件名正则 `T\d+`
   - spec 文件名 stem
3. **行匹配**：grep ID；多匹配时优先选含 markdown 链接指向 spec 的行；仍多匹配 → exit 4（附候选列表）
4. **Marker 替换**（按优先级识别风格）：
   - emoji：`⬜🟨✅❌`（map: pending=⬜ / in-progress=🟨 / done=✅ / cancelled=❌）
   - checkbox：`[ ]` / `[x]`（仅 pending↔done）
   - key-value：`status:\s*\w+`
   - 都不命中 → exit 4
5. **写回**：只替换 marker 子串；写临时文件后 `os.replace` 原子替换

**输出**：stdout JSON `{"path":"...","line":N,"before":"⬜","after":"✅"}`；stderr JSON on error。

**Exit codes**：0 ok / 2 索引缺失 / 3 已在目标状态（idempotent） / 4 歧义或风格未知 / 5 spec 缺失

---

### 2. `~/.claude/custom-harness/bin/review/parse-review.py` + 输出契约

**追加到 `~/.claude/custom-harness/agents/code-reviewer.md` 末尾**：

```
## Output Contract (MUST)

Final lines of response MUST be EXACTLY:

<REVIEW_RESULT>
{"verdict":"pass|fail","crit":N,"maj":N,"min":N,"info":N,
 "findings":[{"sev":"CRITICAL|MAJOR|MINOR|INFO","loc":"path:line","msg":"..."}]}
</REVIEW_RESULT>

- verdict = "fail" iff crit > 0 OR maj > 0
- Strict JSON. No trailing commas. UTF-8.
- Block must be LAST; nothing after </REVIEW_RESULT>.
```

**`~/.claude/custom-harness/bin/review/parse-review.py` 接口**：

```
python ~/.claude/custom-harness/bin/review/parse-review.py [--file <path>]
```

1. `re.search(r'<REVIEW_RESULT>\s*(\{.*?\})\s*</REVIEW_RESULT>', txt, re.DOTALL)` 抽块
2. `json.loads` + schema 校验
3. stdout 输出 canonical JSON（不含外层 tag）
4. exit 0 verdict=pass / exit 1 verdict=fail / exit 5 缺块或畸形

**编排器用法**（替代 LLM 数严重度）：
```bash
# 将 reviewer subagent 输出保存到文件
python ~/.claude/custom-harness/bin/review/parse-review.py --file review-out.txt
# exit 0 → 跳出循环; exit 1 → 继续修复; exit 5 → reviewer 未遵守契约，报错
```

> **Stretch（建议同批做）**：`parse-review.py` 加 `--tag IMPL_RESULT` 选项，复用于解析 `code-implementer` 输出的 `<IMPL_RESULT>` 块（提取 `changed_files`），统一改名 `parse-result.py`。

**`~/.claude/custom-harness/agents/code-implementer.md` 末尾追加**：
```
## Output Contract (MUST)

<IMPL_RESULT>
{"success":true|false,"changed_files":["path/a.py","path/b.py"],
 "tests":{"passed":N,"failed":M},"diff_lines":K}
</IMPL_RESULT>
```

---

### 3. `~/.claude/custom-harness/bin/dispatch.py` envelope

**当前输出**：
- 单条 → 裸 object `{...}`
- `--parallel` → 裸 array `[{...},{...}]`

**改后**：统一

```json
{ "mode": "single",   "payloads": [ {...} ] }
{ "mode": "parallel", "payloads": [ {...}, {...} ] }
```

**`main()` 改动**：末尾两个 `print(json.dumps(...))` 包一层 envelope，`mode` 按 `--parallel` flag 设定。

**`~/.claude/skills/dispatch/SKILL.md` Flow#3 改写**：
```
3. Read envelope:
   - mode="single"   → call Agent tool once with payloads[0]
   - mode="parallel" → in ONE message, call Agent tool once per payload
```

---

### 4. `~/.claude/custom-harness/bin/vsc-commit.py` gitmoji 校验

**新增（脚本顶部）**：
```python
import re as _re

_GITMOJI_RE = _re.compile(
    r'^(✨|🐛|📝|🎨|♻️|⚡|✅|📦|👷|🔧|🔥|🚧)\s+'
    r'\w+(\([^)]+\))?:\s+.+$'
)

def validate_msg(msg: str, vcs: str) -> None:
    if vcs == "git":
        if not _GITMOJI_RE.match(msg):
            die("invalid git msg; expected '<gitmoji> <type>(<scope>): <desc>'", 5)
    else:
        if "\n" in msg.strip() or not (10 <= len(msg) <= 100):
            die("invalid svn msg; expected single line, 10–100 chars", 5)
```

**集成点**：`do_git()` / `do_svn()` 入口第一行调 `validate_msg(opts["msg"], vcs)`。

**`~/.claude/custom-harness/agents/vsc-committer.md` 改动**：删除 Step 2 "Message MUST begin with emoji — regenerate if not"；改为"脚本校验消息格式；exit 5 则重新生成后再调"。

---

## 验证方案

```bash
# 1. task-status.py
printf '⬜ T18 sample\n' > /tmp/TASKS.md
printf -- '---\ntask: T18\n---\n' > /tmp/T18-x.md
python ~/.claude/custom-harness/bin/dev-cycle/task-status.py \
  --spec /tmp/T18-x.md --to done --tasks-index /tmp/TASKS.md
grep '✅ T18' /tmp/TASKS.md

# 2. parse-review.py — 有契约
printf '<REVIEW_RESULT>\n{"verdict":"pass","crit":0,"maj":0,"min":1,"info":0,"findings":[]}\n</REVIEW_RESULT>' \
  | python ~/.claude/custom-harness/bin/review/parse-review.py --file /dev/stdin
# expect exit 0, stdout valid JSON

# 2b. parse-review.py — 无契约
echo "no block" | python ~/.claude/custom-harness/bin/review/parse-review.py --file /dev/stdin
# expect exit 5

# 3. dispatch.py envelope
python ~/.claude/custom-harness/bin/dispatch.py code-reviewer "test" | python -c "import sys,json; d=json.load(sys.stdin); assert d['mode']=='single'"
python ~/.claude/custom-harness/bin/dispatch.py --parallel "code-reviewer a" "code-implementer b" | python -c "import sys,json; d=json.load(sys.stdin); assert d['mode']=='parallel' and len(d['payloads'])==2"

# 4. vsc-commit.py validation
python ~/.claude/custom-harness/bin/vsc-commit.py . -m "bad msg" --dry-run   # expect exit 5
python ~/.claude/custom-harness/bin/vsc-commit.py . -m "✨ feat(x): add y" --dry-run  # expect ok

# 5. 集成回归
# /dispatch dev-cycle <some-spec.md>  — 确认 TASKS 自动更新、循环由 exit code 控制
```

---

## 实现顺序

1. `dispatch.py` envelope + `SKILL.md` flow（改动最小，最先可验证）
2. `vsc-commit.py --validate-msg` + `vsc-committer.md`
3. `bin/review/parse-review.py` + `code-reviewer.md` / `code-implementer.md` 契约
4. `bin/dev-cycle/task-status.py` + `dev-cycle.md` Step 4 改写
5. `dev-cycle.md` / `bug-fixer.md` Step 3/5 接入 `parse-review.py`
6. 端到端 dev-cycle 回归

---

## 留待后续

- `--parallel-from <file.json>`：治理 `--parallel` 跨 shell 引号嵌套问题
- `taskdoc-next.py`：稳定 `doc-designer` 命名/序号推断
- PyYAML 强依赖 / TOML 迁移：消除 `dispatch.py` mini-parser fallback
- `bug-fixer` 子-子 agent 并行编辑冲突：需文件锁/diff 合并，另立 ticket
