# Ultra Compression Rules

Built-in ultra-level rules from doc-refine. Apply at write time, not after.

## Universal

- Preserve: facts, steps, warnings, IDs, code blocks, emphasis.
- Keep tech terms exact: APIs, vars, cmds, paths, errors.
- Doubt → keep.
- Suspend compression for: security warnings, irreversible confirmations, multi-step sequences where compression → misread risk. Resume after.

## CN Ultra (文言文)

全文以极简文言直书。一字能尽不用两字。

### 词汇

| 白话 | 文言 |
|------|------|
| 如果/假如 | 若/如 |
| 所以/因此 | 故/是以 |
| 可以/能够 | 可/能 |
| 应当/应该 | 宜/当 |
| 使用/利用 | 用/以 |
| 修改/更改 | 更/改 |
| 全部/所有 | 皆/尽 |
| 之后/以后 | 既/后 |
| 不需要/不用 | 无须/免 |
| 这个/那个 | 此/其 |
| 但是/不过 | 然/但 |
| 或者/或是 | 或 |
| 并且/而且 | 且/并 |
| 关于/对于 | 论/于 |
| 需要/必须 | 须/必 |
| 通过/借助 | 借/由 |
| 完成/实现 | 成/就 |
| 获取/取得 | 得/取 |
| 删除/移除 | 删/去 |
| 添加/增加 | 增/添 |
| 创建/建立 | 立/建 |
| 返回/回应 | 返/报 |
| 设置/配置 | 设/置 |
| 检查/验证 | 验/察 |
| 显示/展示 | 示/显 |

### 句法

- **省主语**：主语可推断时省之。例："系统将自动检测" → "自检"。
- **判断句**：用"…也"。例："这是核心模块" → "核心模块也"。
- **被动**：用"为…所…"。例："被系统拒绝" → "为系统所拒"。
- **否定**：用"不/无/未/非"。例："不能使用" → "不可用"。
- **并列**：用"、"分隔。例："读取、解析、验证数据"。
- **因果**：用"故/是以/遂"。例："配置错误，所以启动失败" → "配置有误，故启动不成"。
- **箭头化**：因果关系用 X → Y。例："不合→报错"。
- **词组化**：能缩为词组则缩之。例："连接池复用连接" → "池=复用DB连接"。
- 代码块、API名、变量名、命令、路径、错误消息保留原文不译。

### 范例

原文：本模块主要负责对用户输入的数据进行有效性验证，确保数据符合预定义的格式要求，如果验证失败则返回相应的错误信息。
Ultra：验输入格式。不合→报错。

原文：为了提高系统的性能，我们需要对数据库的查询进行优化，主要是通过添加索引和减少不必要的联表查询来实现。
Ultra：优化DB查询→提速。法：增索引、减联表。

原文：连接池会复用已打开的数据库连接，而不是每个请求都新建连接，这样能减少握手开销。
Ultra：池=复用DB连接。免新建→省握手。

## EN Ultra (Caveman)

Write terse. Technical substance stays. Only fluff dies.

### Rules

- Drop: articles (a/an/the), filler (just/really/basically/actually/simply), pleasantries, hedging.
- Fragments OK.
- Short synonyms (big not extensive, fix not "implement a solution for").
- Technical terms exact. Code blocks unchanged. Errors quoted exact.
- Abbreviate: DB/auth/config/req/res/fn/impl.
- Strip conjunctions.
- Arrows for causality: X → Y.
- One word when one word enough.
- Pattern: `[thing] [action] [reason]. [next step].`

### Cut Patterns

| Verbose | Terse |
|---------|-------|
| In order to | to |
| It should be noted that X | X |
| make a decision | decide |
| is used to configure | configures |
| it may be possible to | you can |
| each and every | every |
| a large number of | many |
| due to the fact that | because |
| at this point in time | now |
| has the ability to | can |

### Examples

Before: In order to be able to run the application successfully, it is necessary that you first make sure that you have installed all of the required dependencies that are listed in the requirements file.
Ultra: `pip install -r requirements.txt`. Run app.

Before: Why React component re-render?
Ultra: Inline obj prop → new ref → re-render. `useMemo`.
