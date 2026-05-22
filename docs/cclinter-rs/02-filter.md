# 02 - 过滤器系统

## 核心数据结构

```rust
pub struct FilterSet {
    filters: Vec<String>,      // ["-whitespace/tab", "+build/include_alpha"]
    verbose_level: u8,         // default 1
}
```

## 默认过滤器

```rust
impl Default for FilterSet {
    fn default() -> Self {
        Self {
            filters: vec!["-build/include_alpha".to_string()],
            verbose_level: 1,
        }
    }
}
```

默认抑制 `build/include_alpha`（与 cpplint 一致）。

## API

```rust
impl FilterSet {
    pub fn new() -> Self;                          // 无过滤器, verbose=1
    pub fn add(&mut self, filter: &str);           // "+x" / "-x"
    pub fn add_many(&mut self, filters: &str);     // 逗号分隔 "+x,-y,..."
    pub fn should_print(&self, category: ErrorCategory, confidence: u8) -> bool;
    pub fn backup(&self) -> Self;
    pub fn restore(&mut self, backup: Self);
    pub fn set_verbose(&mut self, level: u8);
}
```

## 过滤逻辑

`should_print(category, confidence)`:

1. 若 `confidence < verbose_level` → `false`
2. 遍历 `filters`，左到右:
   - `"-PREFIX"`: 若 `category.name().starts_with(PREFIX)` → `filtered = true`
   - `"+PREFIX"`: 若 `category.name().starts_with(PREFIX)` → `filtered = false`
3. 最后匹配的 filter 生效

## 示例

```
filters = ["-whitespace", "+whitespace/tab"]

build/include      → + (无匹配 filter)
whitespace/tab     → + (被 +whitespace/tab 覆盖)
whitespace/braces  → - (被 -whitespace 匹配)
```

## C 文件全局抑制

当文件包含 `LINT_C_FILE` 或 `vim: ... filetype=c` 时，全局抑制:
- `readability/casting`

**管线集成点**: 在 `ProcessFileData` 中，`CheckForCopyright` 之后、`RemoveMultiLineComments` 之前检测 marker。若检测到，向 `FilterSet` 追加 `-readability/casting`。使用 `FilterSet::backup()` / `FilterSet::restore()` 限定作用域为该文件。

## Kernel 文件全局抑制

当文件包含 `LINT_KERNEL_FILE` 时，全局抑制:
- `whitespace/tab`

**管线集成点**: 同 C 文件抑制，在同一阶段检测。若检测到，追加 `-whitespace/tab`。

## NOLINT 抑制

```rust
pub struct NolintSuppression {
    pub category: Option<ErrorCategory>,   // None = 抑制全部
    pub linenum: usize,
}
```

- `// NOLINT` — 抑制当前行所有类别
- `// NOLINT(*)` — 同上
- `// NOLINT(category)` — 抑制当前行特定类别
- `// NOLINTNEXTLINE` — 抑制下一行

解析函数:
```rust
fn parse_nolint(raw_line: &str, linenum: usize) -> Vec<NolintSuppression>;
```

未知类别产生 `readability/nolint` 警告，除非是 `clang-analyzer` 前缀（忽略）。

遗留类别静默忽略（不在 `ErrorCategory` enum 中，但 NOLINT 中允许使用）：
- `build/class`
- `readability/streams`
- `readability/function`
