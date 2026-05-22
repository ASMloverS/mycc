# 03 - CleansedLines 预处理管线

## 核心数据结构

```rust
pub struct CleansedLines {
    raw_lines: Vec<String>,                    // 原始行
    lines_without_raw_strings: Vec<String>,    // raw string 替换后
    lines: Vec<String>,                        // 去注释后
    elided: Vec<String>,                       // 去注释 + 字符串折叠后
}
```

4 个等长 `Vec<String>`，行号 1-indexed（index 0 是 marker line）。

## 预处理步骤

### 1. Marker Lines

在文件头尾插入 marker:
```
"// marker so line numbers and indices both start at 1"
"// marker so line numbers end in a known way"
```

### 2. RemoveMultiLineComments

- 扫描 `/* ... */` 跨行注释
- 替换内容为 `/**/` (保留行数)
- 未闭合的注释报 `readability/multiline_comment`

### 3. CleanseRawStrings

替换 C++11 raw string literals `R"delim(...)delim"` → `""`:
```
R"(hello)"      → ""
R"delim(hello
world)delim"    → ""  (跨行)
```

### 4. CleanseComments

对每行:
- 去除 `//` 行尾注释（检查不在字符串内）
- 去除行内 `/* ... */`

### 5. CollapseStrings (生成 elided)

对去注释后的每行:
- `"..."` → `""`
- `'...'` → `''`
- 保留 digit separator: `1'000` 不折叠

## 关键正则

```rust
// C++ escape sequences
static RE_CLEANSE_LINE_ESCAPES: &str = r#"\\([abfnrtv?"\\]|[0-7]{1,3}|x[0-9a-fA-F]{2,4}|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})"#;

// Single-line C comment (multi-line, non-greedy)
static RE_C_COMMENTS: &str = r#"/\*[\s\S]*?\*/"#;

// Include line
static RE_INCLUDE: &str = r#"^\s*#\s*include\s+([<"])([^>"]*)[>"]"#;
```

## API

```rust
impl CleansedLines {
    pub fn from_source(source: &str) -> Self;
    pub fn raw_lines(&self) -> &[String];
    pub fn lines_without_raw_strings(&self) -> &[String];
    pub fn lines(&self) -> &[String];       // 无注释
    pub fn elided(&self) -> &[String];      // 无注释 + 字符串折叠
    pub fn len(&self) -> usize;
}
```
