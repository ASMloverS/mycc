# 04 - 嵌套状态追踪

## 数据结构

使用 enum-based 设计，避免 struct/trait 冲突：

```rust
pub enum BlockKind {
    Class(ClassInfo),
    Namespace(NamespaceInfo),
    ExternC(ExternCInfo),
}
```

### 公共字段提取

每个变体独立持有全部所需字段，`BlockKind` 提供 accessor 方法访问公共字段：

```rust
impl BlockKind {
    pub fn starting_linenum(&self) -> usize { ... }
    pub fn seen_open_brace(&self) -> bool { ... }
    pub fn set_seen_open_brace(&mut self, v: bool) { ... }
    pub fn open_parentheses(&self) -> usize { ... }
    pub fn set_open_parentheses(&mut self, n: usize) { ... }
    pub fn inline_asm(&self) -> AsmState { ... }
    pub fn set_inline_asm(&mut self, s: AsmState) { ... }
    pub fn is_class(&self) -> bool { matches!(self, BlockKind::Class(_)) }
    pub fn is_namespace(&self) -> bool { matches!(self, BlockKind::Namespace(_)) }
}
```

## AsmState

```rust
pub enum AsmState {
    NoAsm,
    InsideAsm,
    EndAsm,
    BlockAsm,
}
```

## ClassInfo

```rust
pub struct ClassInfo {
    pub starting_linenum: usize,
    pub seen_open_brace: bool,
    pub open_parentheses: usize,
    pub inline_asm: AsmState,
    pub name: String,
    pub access: String,        // "public", "protected", "private"
    pub is_derived: bool,
    pub is_struct: bool,
    pub class_indent: usize,
    pub last_line: usize,
}
```

## NamespaceInfo

```rust
pub struct NamespaceInfo {
    pub starting_linenum: usize,
    pub seen_open_brace: bool,
    pub open_parentheses: usize,
    pub inline_asm: AsmState,
    pub name: String,
}
```

## ExternCInfo

```rust
pub struct ExternCInfo {
    pub starting_linenum: usize,
    pub seen_open_brace: bool,
    pub open_parentheses: usize,
    pub inline_asm: AsmState,
}
```

## PreprocessorSnapshot

保存 `#if`/`#ifdef`/`#ifndef` 处的完整栈快照（而非单个 BlockKind）：

```rust
pub struct PreprocessorSnapshot {
    pub stack: Vec<BlockKind>,         // 完整栈克隆
    pub stack_before_else: Vec<BlockKind>,
    pub seen_else: bool,
}
```

`#if` 时 clone 整个 stack，`#else` 时恢复 + 重保存，`#endif` 时恢复。这与 cpplint 的 `_cpplint_state` 行为一致。

## NestingState

```rust
pub struct NestingState {
    stack: Vec<BlockKind>,
    pp_stack: Vec<PreprocessorSnapshot>,
    previous_stack_top: Option<BlockKind>,  // clone, 非 usize 索引
}
```

### API

```rust
impl NestingState {
    pub fn update(&mut self, line: &str, linenum: usize);
    pub fn innermost_class(&self) -> Option<&ClassInfo>;
    pub fn innermost_namespace(&self) -> Option<&NamespaceInfo>;
    pub fn in_class(&self) -> bool;
    pub fn in_namespace(&self) -> bool;
    pub fn in_extern_c(&self) -> bool;
    pub fn in_asm_block(&self) -> bool;
    pub fn stack_top(&self) -> Option<&BlockKind>;
    pub fn previous_stack_top(&self) -> Option<&BlockKind>;
    pub fn check_completed_blocks(&self, filename: &str) -> Vec<Violation>;
}
```

### Update 逻辑

每行调用 `update()`:
1. 保存 `previous_stack_top`（clone 当前栈顶）
2. 检测 `asm` / `__asm__` 块起始
3. 追踪 `(` 计数（inline asm 内）
4. 检测 `class` / `struct` 声明 → push `BlockKind::Class(ClassInfo { ... })`
5. 检测 `namespace` 声明 → push `BlockKind::Namespace(NamespaceInfo { ... })`
6. 检测 `extern "C"` → push `BlockKind::ExternC(ExternCInfo { ... })`
7. 追踪 `{` / `}` 计数
8. `}` 时 pop 栈顶（若 open brace 已闭合）
9. 预处理器 `#if` / `#ifdef` / `#ifndef` → clone 整个 stack 保存为 `PreprocessorSnapshot`
10. `#else` / `#elif` → 恢复 snapshot + 重保存
11. `#endif` → 恢复 snapshot

### Inline ASM 检测正则

```rust
static RE_ASM: &str = r#"^\s*(?:asm|_asm|__asm|__asm__)(?:\s+(volatile|__volatile__))?\s*[{(]"#;
```

### CheckCompletedBlocks

文件末尾检查:
- 若 class 未闭合 → `build/namespaces` 错误（`build/class` 是遗留类别，已合并）
- 若 namespace 未闭合 → `build/namespaces` 错误
- namespace 结尾注释检查（`// namespace foo`）
