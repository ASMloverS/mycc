# 07 - 输出格式

## 支持格式

| 格式 | 标识 | 输出目标 |
|------|------|---------|
| emacs | `emacs` | stderr |
| Visual Studio | `vs7` | stderr |
| Eclipse | `eclipse` | stderr |
| JUnit XML | `junit` | stderr (完整 XML) |
| GNU sed | `sed` | stdout |
| GNU sed (macOS) | `gsed` | stdout |

## 格式规范

### emacs (默认)

```
{filename}:{linenum}: {message} [{category}] [{confidence}]
```

示例:
```
src/main.cc:42: Tab found; replace by spaces [whitespace/tab] [5]
```

### vs7

```
{filename}({linenum}): error cpplint: [{category}] {message} [{confidence}]
```

示例:
```
src/main.cc(42): error cpplint: [whitespace/tab] Tab found; replace by spaces [5]
```

### eclipse

```
{filename}:{linenum}: warning: {message}  [{category}] [{confidence}]
```

示例:
```
src/main.cc:42: warning: Tab found; replace by spaces  [whitespace/tab] [5]
```

### junit

完整 JUnit XML:
```xml
<?xml version="1.0" encoding="UTF-8" ?>
<testsuite errors="0" failures="2" name="cpplint" tests="2">
  <testcase name="src/main.cc">
    <failure>42: Tab found [whitespace/tab] [5]
48: Line too long [whitespace/line_length] [5]</failure>
  </testcase>
</testsuite>
```

实现: 按文件分组 failures，使用 `quick-xml` 生成。

**零 violation 文件处理**: 无 violation 的文件不出现在 `<testsuite>` 中（即 `tests` 计数 = 有 violation 的文件数）。这与 cpplint 行为一致。

### sed / gsed

输出 sed 命令到 stdout，用于自动修复特定问题。

**sed vs gsed 差异**:
- `sed` 模式: 输出 GNU sed 命令 `sed -i 'EXPR' FILE`（适用于 Linux）
- `gsed` 模式: 输出 BSD sed 兼容命令 `sed -i '' 'EXPR' FILE`（适用于 macOS）
  - macOS 自带的 sed 是 BSD 版本，`-i` 要求紧跟空字符串参数
  - `gsed` 是通过 Homebrew 安装的 GNU sed 的别名

不可自动修复的问题输出注释到 stderr:
```
# 42: Cannot auto-fix: Tab found [whitespace/tab] [5]
```

修复映射:

| 消息关键词 | sed 命令 |
|-----------|---------|
| spaces around = | `s/ = /=/` |
| spaces around != | `s/ != /!=/` |
| Tab found | `s/\t/  /g` |
| Line ends in whitespace | `s/\s*$//` |
| ; after } | `s/};/}/` |
| Missing space after , | `s/,([^ ])/, \1/g` |

## 统计输出

文件处理完成后打印统计:

```
Category 'whitespace/tab' errors found: 3
Category 'build/include' errors found: 1
Total errors found: 4
```

模式由 `--counting` 控制（见 10-cli.md `Cli.counting` 字段）:
- `total`: 仅打印总数
- `toplevel`: 按顶级分类 (whitespace/build/...)
- `detailed`: 按完整类别 (whitespace/tab/build/include/...)

## 静默模式

`--quiet`: 无错误时不输出任何内容。
