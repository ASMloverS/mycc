# 11 - 实施计划

## 总览

| Phase | Tasks | 内容 | 预估 LOC |
|-------|-------|------|----------|
| 1 | T1-T10 | 基础设施 | ~2500 |
| 2 | T11-T15 | 全部 66 检查函数 | ~3000 |
| 3 | T16 | 扩展检查 + Auto-fix | ~500 |
| 4 | T17-T19 | 编排 + CLI + 并行 | ~800 |
| 5 | T20-T22 | NOLINT + 对标测试 + 文档 | ~500 |
| **合计** | **22** | | **~7300** |

---

## Phase 1: Foundation ✅

> **状态**: 已完成 (2026-05-22)。75 个测试通过，0 clippy 警告。
> **代码**: `tools/linter/cclinter-rs/` (~2500 LOC)

### T1: 项目初始化 ✅

- 创建 `Cargo.toml`（依赖：clap, regex, rayon, serde, toml, quick-xml, walkdir, globset, criterion, thiserror）
- `src/main.rs`, `src/lib.rs`
- 验证 `cargo build` 通过

### T2: 错误类型 + ErrorCategory 系统 ✅

- 定义 `LintError` enum（`thiserror` 派生）
- 实现 70 个变体的 enum（66 原始 cpplint 2.0.2 + 4 扩展）
  - build: 17, legal: 1, readability: 14, runtime: 15, whitespace: 19, extensions: 4
- `Display`, `FromStr`, `group()`, `all()`
- `Violation` struct
- 测试: roundtrip, all_70, unknown_parse_fail, group

### T3: Filter 系统 ✅

- `FilterSet` struct
- 默认过滤器 `["-build/include_alpha"]`
- `add()`, `should_print()`, `backup()`, `restore()`
- 测试: default, positive, negative, prefix, order, confidence

### T4: 配置系统 (TOML) ✅

- `Config` struct (serde Deserialize)
- `FilterConfig`, `ExtensionConfig`, `ExcludeConfig`, `FixConfig`
- `load()`, `with_cli_overrides(self, cli)`（统一方法名）
- `effective_*()` 方法含 fallback 默认值
- 测试: parse, defaults, search

### T5: CleansedLines 预处理 ✅

- 4 层行数组: raw, without_raw_strings, without_comments, elided
- `RemoveMultiLineComments`, `CleanseRawStrings`, `CleanseComments`, `CollapseStrings`
- 测试: parallel_arrays, comments_stripped, strings_collapsed, raw_strings, multiline

### T6: NestingState ✅

- `BlockKind` enum (`Class(ClassInfo)`, `Namespace(NamespaceInfo)`, `ExternC(ExternCInfo)`)
- `ClassInfo`, `NamespaceInfo`, `ExternCInfo` 各自独立 struct（公共字段独立持有）
- `PreprocessorSnapshot`（完整栈克隆，非单个 BlockKind）
- `NestingState` with `update()`
- `previous_stack_top: Option<BlockKind>`（clone）
- 测试: nesting_level, class_detect, namespace, extern_c, preprocessor, asm

### T7: FileInfo ✅

- 路径工具: `RepositoryName`, `Split`, header guard 推导
- 独立测试

### T8: Headers 数据集 ✅

- `CPP_HEADERS`, `C_HEADERS`, `C_STANDARD_HEADER_FOLDERS`
- `classify_include()` 函数
- `LazyLock<HashSet>` 存储（Rust 1.80+）
- `HEADERS_CONTAINING_TEMPLATES` 映射（从 cpplint 移植 ~60 条）

### T9: IncludeState + FunctionState ✅

- `IncludeState`: include 排序追踪, 字母序检查
- `FunctionState`: 函数体行数追踪
- 各自独立测试

### T10: LintContext + Output ✅

- `LintContext<'a>` 组合所有 per-file 状态
- `report()` 方法（含 NOLINT 抑制检查 + filter 检查）
- `is_suppressed()` 方法
- 6 种输出格式函数
- 测试: emacs, vs7, eclipse, junit, sed 格式

---

## Phase 2: Check Functions ✅

> **状态**: 已完成 (2026-05-23)。184 个测试通过，0 clippy 警告。
> **代码**: `tools/linter/cclinter-rs/src/checks/` (~2500 LOC)

### T11: whitespace/* (19 类别) ✅

- `src/checks/whitespace.rs`: 17 个检查函数
- check_tab, check_indent, check_indent_namespace, check_end_of_line, check_line_length
- check_braces, check_blank_line, check_comma, check_semicolon, check_comments
- check_operators, check_parens, check_empty_body, check_newline, check_ending_newline
- check_forcolon, check_todo
- Unicode 宽度计算（CJK 字符算 2 宽度）
- URL 注释行豁免行长度检查
- 测试: 25 个

### T12: build/* (17 类别) ✅

- `src/checks/build.rs`: 15 个检查函数
- check_header_guard, check_include_order, check_include, check_include_what_you_use
- check_namespaces, check_namespaces_headers, check_namespaces_literals
- check_cpp11, check_cpp17, check_deprecated, check_endif_comment
- check_explicit_make_pair, check_printf_format, check_storage_class, check_forward_decl
- 测试: 18 个

### T13: readability/* (14 类别) ✅

- `src/checks/readability.rs`: 14 个检查函数
- check_casting, check_constructors, check_fn_size, check_braces_readability
- check_strings, check_todo_readability, check_namespace_readability, check_alt_tokens
- check_check, check_inheritance, check_multiline_comment, check_nolint, check_nul, check_utf8
- 测试: 19 个

### T14: runtime/* (15 类别) ✅

- `src/checks/runtime.rs`: 15 个检查函数
- check_references, check_string, check_printf, check_printf_format, check_int
- check_explicit, check_casting, check_memset, check_init, check_operator
- check_arrays, check_invalid_increment, check_member_string_references, check_threadsafe_fn, check_vlog
- 测试: 22 个

### T15: legal/copyright (1 类别) ✅

- `src/checks/legal.rs`: check_copyright()
- 前 10 行检查 "Copyright"（大小写不敏感），含年份验证
- 测试: 11 个

---

## Phase 3: Extensions ✅

> **状态**: 已完成 (2026-05-23)。232 个测试通过，0 clippy 警告。
> **代码**: `tools/linter/cclinter-rs/src/checks/extensions.rs` + `src/fix.rs` (~450 LOC)

### T16: 扩展检查 + Auto-fix ✅

1. block_comment 检测（使用 raw_lines） ✅
2. utf8_bom 检测（字节级） ✅
3. utf8_invalid 检测（字节级） ✅
4. crlf 检测（字节级，`&[u8]` 签名） ✅
5. fix_trailing_whitespace ✅
6. fix_utf8_bom（字节级） ✅
7. fix_crlf（字节级） ✅
8. fix_block_to_line_comments（使用 CleansedLines 比较定位真实注释，处理字符串内误转） ✅
9. FixEngine 组合 ✅

---

## Phase 4: Orchestration + CLI

### T17: ProcessFile + ProcessFileData

- 完整管线实现
- 全局抑制检测（C/Kernel 文件 marker → 追加 filter）
- 集成测试（fixture 文件）

### T18: CLI (clap)

- 参数定义（含 `--counting`）
- main 函数（lint → 输出 → fix 顺序）
- `--init` 功能（文件已存在时报错退出码 2）

### T19: 并行处理

- rayon par_iter
- criterion 基准测试

---

## Phase 5: Integration + Polish

### T20: NOLINT 抑制

- NOLINT / NOLINTNEXTLINE 解析
- 遗留类别兼容（`build/class`, `readability/streams`, `readability/function`）
- 集成到 LintContext

### T21: Fixture 对标测试

- 从 cpplint_unittest.py 移植测试
- 对比 cpplint 和 cclinter-rs 输出

### T22: 示例配置

- 完整 `cclinter-rs.toml` 示例

---

## 提交规范

```
gitmoji type(scope): desc

✨ feat(error): add ErrorCategory enum
🐛 fix(whitespace): fix tab detection in raw strings
✅ test(build): add include_order tests
⚡ perf(process): add rayon parallel processing
```

## 验证检查点

每个 Phase 完成后:
1. `cargo build` — 编译通过
2. `cargo test` — 全部测试通过
3. `cargo clippy` — 无警告
4. 手动对比 Python cpplint 输出（Phase 4 之后）
