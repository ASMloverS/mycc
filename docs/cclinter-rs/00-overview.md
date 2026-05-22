# cclinter-rs: 设计概述

> Rust 复刻 cpplint 2.0.2 + 性能优化 + 扩展规则

## 目标

将 Python cpplint (6860 行, 66 个错误类别) 完整复刻为 Rust CLI 工具。

新增扩展规则：
- 禁止 `/* */` 块注释（仅允许 `//`）
- UTF-8 编码强制
- LF 换行强制
- 行尾空白符检测 + auto-fix
- `/* */` → `//` 自动转换

针对大规模代码库（10k+ 文件）使用 rayon 并行处理。

## 技术栈

| 项目 | 选择 |
|------|------|
| 语言 | Rust 2021 edition, MSRV 1.80+ |
| CLI | clap (derive) |
| 正则 | regex |
| 并行 | rayon |
| 配置 | serde + toml |
| XML 输出 | quick-xml |
| 文件遍历 | walkdir |
| Glob 匹配 | globset |
| 基准测试 | criterion |

> **MSRV 说明**: `std::sync::LazyLock` 在 Rust 1.80 稳定，是本项目的最低要求。不使用 Rust 2024 edition 以保持 1.80 兼容性。

## 配置

使用 TOML 格式（`cclinter-rs.toml`），不兼容 Python cpplint 的 `CPPLINT.cfg`。

**迁移说明**: `CPPLINT.cfg` 采用 `key=value` 逐行格式，`cclinter-rs.toml` 使用标准 TOML。迁移时需手动转换：`filter=-x,+y` → `[filters] add = ["-x", "+y"]`；`linelength=120` → `line_length = 120`。`--init` 可生成默认配置文件作为起点。

详见 [06-config.md](06-config.md)。

## 输出格式

全部复刻 cpplint 的 6 种格式：emacs / vs7 / eclipse / junit / sed / gsed。

详见 [07-output.md](07-output.md)。

## 错误处理策略

统一错误类型：

```rust
#[derive(Debug, thiserror::Error)]
pub enum LintError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("{0}")]
    InvalidArgs(String),
}
```

文件级错误（权限拒绝、编码无效等）打印到 stderr 并跳过该文件，不终止整个 lint 流程。CLI 参数错误 / 配置解析错误立即终止并返回退出码 2。

## 架构

单 workspace Cargo 项目，bin target。

```
cclinter-rs/
├── Cargo.toml
├── cclinter-rs.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── error.rs          # LintError, ErrorCategory, Violation
│   ├── config.rs         # TOML 配置
│   ├── filter.rs         # Category filter
│   ├── file_info.rs      # 路径工具
│   ├── cleanse.rs        # CleansedLines 预处理
│   ├── nesting.rs        # 嵌套追踪 (BlockKind enum)
│   ├── include_state.rs  # include 排序
│   ├── function_state.rs # 函数长度
│   ├── lint_context.rs   # per-file 状态 (见 05a-lint-context.md)
│   ├── process.rs        # 编排层
│   ├── output.rs         # 输出格式化
│   ├── headers.rs        # C/C++ 头文件集合
│   ├── checks/
│   │   ├── mod.rs
│   │   ├── legal.rs
│   │   ├── build.rs
│   │   ├── whitespace.rs
│   │   ├── readability.rs
│   │   ├── runtime.rs
│   │   └── extensions.rs # 扩展检查
│   └── fix.rs            # Auto-fix 引擎
├── tests/
│   ├── fixtures/
│   └── ...
└── benches/
    └── bench_lint.rs
```

## 处理管线

```
文件发现 (--recursive/--exclude/--extensions)
  ↓
ProcessFile
  ├── 读取 (binary, 编码检测)
  ├── CRLF/LF 检测
  └── ProcessFileData
      ├── 前置 marker lines
      ├── NOLINT 抑制解析
      ├── CheckForCopyright
      ├── 全局抑制检测:
      │   ├── LINT_C_FILE / vim: filetype=c → 追加 -readability/casting
      │   └── LINT_KERNEL_FILE → 追加 -whitespace/tab
      ├── RemoveMultiLineComments
      ├── 构建 CleansedLines (4 层行数组)
      ├── header guard 检查
      ├── 逐行检查:
      │   ├── NestingState.Update
      │   ├── CheckStyle (whitespace/*)
      │   ├── CheckLanguage (build/*, runtime/*)
      │   ├── CheckForNonConstReference
      │   ├── CheckForNonStandardConstructs
      │   └── 扩展检查 (使用 raw_lines)
      ├── 后置检查 (include_what_you_use, ending_newline, etc.)
      └── 扩展检查 (UTF-8 BOM, UTF-8 编码, CRLF)
  ↓
Violation 聚合 → FilterSet 过滤 → 输出格式化 → 终端输出
```

## 模块详细设计

| 文档 | 内容 |
|------|------|
| [01-error-category.md](01-error-category.md) | 错误类别 (66 原始 + 4 扩展 = 70) + Violation 类型 |
| [02-filter.md](02-filter.md) | 过滤器系统 + NOLINT 抑制 |
| [03-preprocess.md](03-preprocess.md) | CleansedLines 预处理 |
| [04-nesting.md](04-nesting.md) | 嵌套状态追踪 (BlockKind enum) |
| [05-headers.md](05-headers.md) | 头文件集合 + include 分类 |
| [05a-lint-context.md](05a-lint-context.md) | LintContext + IncludeState + FunctionState + FileInfo |
| [06-config.md](06-config.md) | TOML 配置 |
| [07-output.md](07-output.md) | 输出格式 |
| [08-checks.md](08-checks.md) | 全部 66 + 4 检查规则 |
| [09-extensions.md](09-extensions.md) | 扩展检查 + Auto-fix |
| [10-cli.md](10-cli.md) | CLI 接口 |
| [11-impl-plan.md](11-impl-plan.md) | 实施计划 |

## 性能优化

| 优化点 | 方式 |
|--------|------|
| 多文件并行 | rayon `par_iter` |
| 头文件集合 | `std::sync::LazyLock<HashSet>` |
| 正则预编译 | `std::sync::LazyLock<Regex>` |
| 文件读取 | `std::fs::read` + 手动 UTF-8 验证 |
| Auto-fix | 单次写回 |
| 并行内存控制 | rayon 分批处理，单文件 CleansedLines 处理完毕即释放 |

## 对标

- 行为与 cpplint 2.0.2 一致（66 个错误类别）
- 新增 4 个扩展类别（共 70）
- NOLINT/NOLINTNEXTLINE 抑制兼容
- 输出格式兼容
