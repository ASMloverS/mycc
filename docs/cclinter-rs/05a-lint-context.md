# 05a - LintContext + Per-File 状态

## LintContext

全部检查函数通过 `LintContext` 报告结果和访问共享状态：

```rust
pub struct LintContext<'a> {
    pub filename: String,
    pub lines: &'a CleansedLines,
    pub nesting: NestingState,
    pub filter: &'a FilterSet,
    pub config: &'a Config,
    pub nolint_suppressions: Vec<NolintSuppression>,
    pub violations: Vec<Violation>,
    pub include_state: IncludeState,
    pub function_state: FunctionState,
    pub file_info: FileInfo,
    pub is_c_file: bool,
    pub is_kernel_file: bool,
}

impl<'a> LintContext<'a> {
    pub fn new(
        filename: &str,
        lines: &'a CleansedLines,
        filter: &'a FilterSet,
        config: &'a Config,
    ) -> Self;

    /// 报告 violation，自动检查 NOLINT 抑制和 filter
    pub fn report(&mut self, linenum: usize, category: ErrorCategory, confidence: u8, message: &str);

    /// 检查指定行号和类别是否被 NOLINT 抑制
    pub fn is_suppressed(&self, linenum: usize, category: ErrorCategory) -> bool;

    /// 消费 self，返回收集到的 violations
    pub fn into_violations(self) -> Vec<Violation>;
}
```

### report 逻辑

```rust
fn report(&mut self, linenum: usize, category: ErrorCategory, confidence: u8, message: &str) {
    if self.is_suppressed(linenum, category) {
        return;
    }
    if !self.filter.should_print(category, confidence) {
        return;
    }
    self.violations.push(Violation {
        filename: self.filename.clone(),
        linenum,
        category,
        confidence,
        message: message.to_string(),
    });
}
```

---

## IncludeState

追踪 include 顺序和分组：

```rust
pub struct IncludeState {
    pub section: u8,                    // 当前 include 分组 (1=对应.h, 2=C系统, 3=C++系统, 4=其他系统, 5=项目)
    pub last_include: Option<String>,   // 上一个 include 路径（字母序检查）
    pub last_sorted_section: u8,
    pub include_list: Vec<(String, u8)>, // (path, section)
}

impl IncludeState {
    pub fn new() -> Self;
    pub fn reset(&mut self, section: u8);
    pub fn check_include_order(
        &mut self,
        include_path: &str,
        linenum: usize,
        section: u8,
        filename: &str,
    ) -> Vec<Violation>;
    pub fn check_alpha(&self, include_path: &str) -> Option<Violation>;
}
```

---

## FunctionState

追踪函数体行数：

```rust
pub struct FunctionState {
    pub in_function: bool,
    pub function_name: String,
    pub function_start: usize,   // 1-indexed
    pub function_lines: usize,
    pub is_test_function: bool,
}

impl FunctionState {
    pub fn new() -> Self;
    pub fn begin(&mut self, name: &str, linenum: usize, is_test: bool);
    pub fn end(&mut self) -> Option<Violation>;  // 超过阈值时返回 violation
    pub fn count_line(&mut self);
}
```

阈值：普通函数 250 行，测试函数 400 行（`readability/fn_size`）。

---

## FileInfo

路径工具函数：

```rust
pub struct FileInfo {
    pub filename: String,       // 原始路径
    pub basename: String,       // 文件名部分 (含扩展名)
    pub stem: String,           // 文件名不含扩展名
    pub extension: String,      // 扩展名 (不含 .)
    pub is_header: bool,        // 是否为头文件
    pub repository_name: String, // --repository 去除的前缀
    pub file_extension: String, // 实际扩展名
}

impl FileInfo {
    pub fn new(filename: &str, repository: &str, header_extensions: &HashSet<String>) -> Self;

    /// 推导 header guard 宏名
    /// 规则: 路径 → 大写 → / → _ → . → _ → 加 _H_ 后缀
    /// 示例: "src/foo/bar.h" → SRC_FOO_BAR_H_
    pub fn header_guard(&self, root: &str) -> String;

    /// 获取不带 repository 前缀的相对路径
    pub fn repository_name(&self) -> &str;

    /// 检查是否为 C 标准头文件扩展名
    pub fn is_c_file(&self) -> bool;
}
```

---

## 检查函数签名规范

所有检查函数统一使用 `impl LintContext` 方法形式：

```rust
impl LintContext<'_> {
    // 逐行检查
    fn check_style(&mut self, linenum: usize);
    fn check_language(&mut self, linenum: usize);
    fn check_non_const_reference(&mut self, linenum: usize);
    fn check_non_standard_constructs(&mut self, linenum: usize);
    fn check_extensions(&mut self, linenum: usize);

    // 全文件检查
    fn check_copyright(&mut self);
    fn check_header_guard(&mut self);
    fn check_newline_at_eof(&mut self);
    fn check_include_what_you_use(&mut self);
    fn check_completed_blocks(&mut self);
}
```

不使用自由函数形式，以统一访问 `self` 上的所有状态。
