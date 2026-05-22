# 05 - 头文件集合 + Include 分类

## 数据集

### C++ 标准头文件 (~150 个)

包含:
- Legacy: `algobase.h`, `algo.h`, `alloc.h`, `deque.h`, `hash_map`, ...
- C++11/14/17/20/23: `algorithm`, `array`, `atomic`, `chrono`, `concepts`, `coroutine`, `format`, `generator`, ...

### C 标准头文件 (~120 个)

包含:
- 标准 C: `assert.h`, `ctype.h`, `errno.h`, `stdio.h`, `stdlib.h`, `string.h`, ...
- C23: `stdbit.h`, `stdckdint.h`
- POSIX: `dirent.h`, `pthread.h`, `unistd.h`, ...
- GNU/Linux: `elf.h`, `features.h`, `malloc.h`, ...

### 标准目录

C standard header folders:
- `sys`, `arpa`, `asm-generic`, `bits`, `gnu`, `net`, `netinet`, `protocols`, `rpc`, `rpcsvc`, `scsi`
- Linux kernel: `drm`, `linux`, `misc`, `mtd`, `rdma`, `sound`, `video`, `xen`

## Include 类型常量

```rust
pub const C_SYS_HEADER: u8     = 1;
pub const CPP_SYS_HEADER: u8   = 2;
pub const OTHER_SYS_HEADER: u8 = 3;
pub const LIKELY_MY_HEADER: u8 = 4;
pub const POSSIBLE_MY_HEADER: u8 = 5;
pub const OTHER_HEADER: u8     = 6;
```

## 分类函数

```rust
pub fn classify_include(
    include_path: &str,
    filename: &str,
    include_order: &str,  // "default" | "standardcfirst"
) -> u8;
```

### 分类规则

1. **`LIKELY_MY_HEADER`**: include 路径的 basename 去掉后缀与源文件 basename 去掉后缀匹配
   - `browser.cc` include `"browser.h"` → LIKELY_MY_HEADER
2. **`POSSIBLE_MY_HEADER`**: include 路径的第一段与源文件第一段匹配
   - `chrome/browser.cc` include `"chrome/browser.h"` → POSSIBLE_MY_HEADER
3. **`C_SYS_HEADER`**: 尖括号 include，扩展名 `.h`，且在 `_C_HEADERS` 或 `C_STANDARD_HEADER_FOLDERS` 中
4. **`CPP_SYS_HEADER`**: 尖括号 include，在 `_CPP_HEADERS` 中
5. **`OTHER_SYS_HEADER`**: 尖括号 include，有扩展名（`includeorder=standardcfirst` 模式下非标准 C 头）
6. **`OTHER_HEADER`**: 其余

### 第三方头文件豁免

大写字母文件名（如 `Python.h`, `QtWidgets`）或 Lua 头文件跳过 include 排序检查。

```rust
static RE_THIRD_PARTY: &str = r#"^(?:[^/]*[A-Z][^/]*\.h|lua\.h|lauxlib\.h|lualib\.h)$"#;
```

## 存储方式

使用 `std::sync::LazyLock<HashSet<&'static str>>`（Rust 1.80+ 稳定，与 MSRV 一致）:

```rust
use std::sync::LazyLock;
use std::collections::HashSet;

static CPP_HEADERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "algorithm", "array", "atomic", "bitset", "chrono", ...
    ])
});

static C_HEADERS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "assert.h", "ctype.h", "errno.h", "stdio.h", ...
    ])
});
```

## IncludeWhatYouUse 映射

模板使用 → 需要 include 的头文件。完整映射表从 cpplint `_HEADERS_CONTAINING_TEMPLATES` 移植，约 60 条规则。

示例条目：

| 使用模式 | 需要 include |
|----------|-------------|
| `std::vector` | `<vector>` |
| `std::map` | `<map>` |
| `std::string` | `<string>` |
| `std::set` | `<set>` |
| `std::unordered_map` | `<unordered_map>` |
| `std::unique_ptr` | `<memory>` |
| `std::shared_ptr` | `<memory>` |
| `std::function` | `<functional>` |
| `std::pair` | `<utility>` |
| `std::sort` | `<algorithm>` |
| `std::cout` | `<iostream>` |
| `std::ostringstream` | `<sstream>` |
| `std::thread` | `<thread>` |
| `std::mutex` | `<mutex>` |

```rust
static HEADERS_CONTAINING_TEMPLATES: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
    LazyLock::new(|| {
        HashMap::from([
            ("std::vector", vec!["<vector>"]),
            ("std::map", vec!["<map>"]),
            ("std::string", vec!["<string>"]),
            // ... 完整映射从 cpplint 移植
        ])
    });
```
