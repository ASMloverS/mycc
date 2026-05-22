# 08 - 检查规则 (66 类别)

全部检查函数统一为 `impl LintContext` 方法形式（见 05a-lint-context.md），不使用自由函数：

```rust
impl LintContext<'_> {
    fn check_xxx(&mut self, linenum: usize);
    // 或全文件级:
    fn check_xxx(&mut self);
}
```

---

## 全文件级检查 (ProcessFileData 中调用)

### legal/copyright

`CheckForCopyright`: 前 10 行必须包含 "Copyright"（大小写不敏感）。

### build/header_guard

`CheckForHeaderGuard`: .h 文件必须有 `#ifndef`/`#define`/`#endif` 保护宏。
- 宏名 = 路径转大写 + `_H_` 后缀
- 支持 `#pragma once` 替代
- 检查 `#endif` 后的注释与 guard 名匹配

### build/include

`CheckHeaderFileIncluded`: .cc 文件必须 include 对应的 .h 文件。

### readability/utf8

`CheckForBadCharacters`: 检测 Unicode 替换字符 (U+FFFD) 和 NUL 字节。

### whitespace/ending_newline

`CheckForNewlineAtEOF`: 文件必须以换行符结尾。

### build/include_what_you_use

`CheckForIncludeWhatYouUse`: 扫描使用的 STL 类型，检查是否有对应 `#include`。

### build/namespaces

`CheckCompletedBlocks`: 文件末尾 class 或 namespace 未闭合时报错。
- class 未闭合 → `build/namespaces` 错误
- namespace 未闭合 → `build/namespaces` 错误

> **注**: `build/class` 是 cpplint 遗留类别，已合并到 `build/namespaces`。NOLINT 中仍可使用 `build/class` 作为遗留兼容。

---

## 逐行检查 (ProcessLine 中调用)

### readability/multiline_comment

`CheckForMultilineCommentsAndStrings`: 检测跨行 `/* */` 注释。

### readability/multiline_string

`CheckForMultilineCommentsAndStrings`: 检测跨行字符串字面量。

### whitespace/indent_namespace

`CheckForNamespaceIndentation`: namespace 体内有缩进时报错。

### readability/fn_size

`CheckForFunctionLengths`: 追踪函数体行数，超过阈值 (普通 250 行, 测试 400 行) 报错。

### whitespace/tab

`CheckStyle` 子检查: 检测 Tab 字符。

### whitespace/indent

`CheckStyle` 子检查: 检测 1 或 3 空格缩进（应为 2 的倍数）。

### whitespace/end_of_line

`CheckStyle` 子检查: 检测行尾空白符。

### whitespace/line_length

`CheckStyle` 子检查: 行长度超过限制（默认 80）。Unicode 字符按显示宽度计算。

### whitespace/newline

`CheckStyle` 子检查: 一行内多条语句。文件级检查混合 CRLF/LF。

### whitespace/braces

`CheckBraces`:
- `{` 应在前一行末尾（函数定义除外）
- `} else {` 应在同一行
- `else` 前必须有 `}`
- if/else/for/while 的 `{` 不可省略（单行除外）

### readability/braces

`CheckTrailingSemicolon`: `};` 后多余分号。
`CheckBraces` 中的部分规则。

### whitespace/blank_line

`CheckSpacing`:
- `{` 后不应有空行
- `}` 前不应有空行
- `public:`/`protected:`/`private:` 后应有空行（`CheckSectionSpacing`）

### whitespace/comma

`CheckCommaSpacing`: 逗号后应有空格。

### whitespace/semicolon

`CheckCommaSpacing`: 分号后应有空格（for 循环内除外）。

### whitespace/comments

`CheckComment`:
- `//` 后应有空格
- TODO 格式: `// TODO(username): description`
- `// NOLINT` 格式

### whitespace/operators

`CheckOperatorSpacing`:
- `=` 前后应有空格
- `==`, `!=`, `<=`, `>=` 前后应有空格
- `<<`, `>>` 前后应有空格
- 一元操作符（`!`, `~`, `++`, `--`）后不应有空格

### whitespace/parens

`CheckParenthesisSpacing`: `if(`/`for(`/`while(`/`switch(` 内不应有空格。
`CheckSpacingForFunctionCall`: 函数调用括号内不应有多余空格。

### whitespace/empty_conditional_body

`CheckEmptyBlockBody`: 条件体用 `;` 空实现。

### whitespace/empty_if_body

`CheckEmptyBlockBody`: if 体用 `;` 空实现。

### whitespace/empty_loop_body

`CheckEmptyBlockBody`: for/while 体用 `;` 空实现。

### whitespace/forcolon

`CheckStyle`: range-based for 的 `:` 前后空格检查。

### readability/alt_tokens

`CheckAltTokens`: 使用 `and`/`or`/`not`/`bitor` 等替代 token 时建议使用符号形式。

### readability/check

`CheckCheck`: `CHECK(a == b)` 应改为 `CHECK_EQ(a, b)` 等。

### build/include_order

`CheckIncludeLine`: include 顺序: 对应 .h → C 系统 → C++ 系统 → 其他系统 → 项目头文件。

### build/include_alpha

`CheckIncludeLine`: 同组 include 应按字母序排列。

### build/include_subdir

`CheckIncludeLine`: 项目 include 应使用完整子目录路径。

### build/c++11, build/c++17

`FlagCxx11Features` / `FlagCxx17Features`: 标记 C++11/C++17 头文件和特性。

> **注**: cpplint 2.0.2 移除了 `build/c++14` 和 `build/c++tr1` 类别。

### build/deprecated

`CheckForNonStandardConstructs`: `>?` 和 `<?` 运算符（已弃用）。

### build/endif_comment

`CheckForNonStandardConstructs`: `#endif` 后应跟注释说明对应的 `#if` 条件。

### build/explicit_make_pair

`CheckMakePairUsesDeduction`: `make_pair<T, U>` 不需要显式模板参数。

### build/forward_decl

`CheckForNonStandardConstructs`: 内部风格前向声明 `class A::B;`。

### build/namespaces

`CheckLanguage`: `using namespace` 指令（尤其是头文件中）。

### build/namespaces_headers

`CheckLanguage`: 头文件中的 unnamed namespace。

### build/namespaces_literals

`CheckLanguage`: `using namespace` 用于 literals。

### build/printf_format

`CheckForNonStandardConstructs`: `%q` 格式，`%N$` 格式，未定义转义。

### build/storage_class

`CheckForNonStandardConstructs`: storage-class 修饰符不在声明开头。

### readability/casting

`CheckCasts`: C 风格 cast `(int)x` 应使用 `static_cast<int>(x)`。

### readability/constructors

`CheckForNonStandardConstructs`:
- 单参数构造函数应标记 `explicit`
- 零参数构造函数不应标记 `explicit`

### readability/inheritance

`CheckRedundantVirtual`: `virtual` + `override`/`final` 冗余。
`CheckRedundantOverrideOrFinal`: 同时有 `override` 和 `final` 冗余。

### readability/namespace

namespace 相关缩进和格式检查。

### readability/nolint

NOLINT 注释中使用了未知类别时警告。

### readability/nul

`CheckForBadCharacters`: NUL 字节 (`\0` 不在字符串中时)。

### readability/todo

`CheckComment`: TODO 格式检查 `// TODO(user):`。

### runtime/arrays

`CheckLanguage`: 可变长度数组 (VLA)。

### runtime/casting

`CheckCasts`: `char *` cast 等。

### runtime/explicit

`CheckForNonStandardConstructs`: 同 `readability/constructors` 相关的 explicit 检查。

### runtime/int

`CheckLanguage`: 使用 `short`, `long`, `long long` 而非 `<cstdint>` 类型。

### runtime/init

`CheckGlobalStatic`: 自引用初始化。

### runtime/invalid_increment

`CheckInvalidIncrement`: `*count++` 语义错误（指针递增而非值递增）。

### runtime/member_string_references

`CheckForNonStandardConstructs`: `const string&` 成员变量。

### runtime/memset

`CheckLanguage`: memset 参数顺序错误 (`memset(arr, sizeof(arr), 0)` 应为 `memset(arr, 0, sizeof(arr))`)。

### runtime/operator

`CheckLanguage`: 重载 `operator&`（取地址 vs 按位与）。

### runtime/printf

`CheckPrintf`: 使用 `sprintf`, `strcpy`, `strcat` 等不安全函数。
`CheckLanguage`: printf 格式串漏洞。

### runtime/printf_format

`CheckForNonStandardConstructs`: 格式串问题。

### runtime/references

`CheckForNonConstReference`: 非 const 引用参数。

### runtime/string

`CheckGlobalStatic`: 全局/静态 `std::string`。

### runtime/threadsafe_fn

`CheckPosixThreading`: 使用线程不安全的 POSIX 函数 (asctime, ctime, gmtime, localtime, rand, strtok)。

### runtime/vlog

`CheckVlogArguments`: VLOG() 使用符号名而非数字级别。

### whitespace/todo

TODO 注释格式中的空白问题。
