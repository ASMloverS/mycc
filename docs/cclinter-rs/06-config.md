# 06 - TOML 配置系统

## 配置文件名

`cclinter-rs.toml`

## 搜索策略

从被 lint 的文件所在目录向上搜索，找到的第一个 `cclinter-rs.toml` 生效。
不继承合并（简化设计，与 cpplint 的 CPPLINT.cfg 向上累积不同）。

## 完整结构

```toml
# cclinter-rs.toml
verbose = 1                          # 0-5, default 1
line_length = 80                     # default 80
output = "emacs"                     # emacs|vs7|eclipse|junit|sed|gsed
recursive = false                    # default false
quiet = false                        # default false
repository = ""                      # repo root path
root = ""                            # header guard root subdir
include_order = "default"            # default|standardcfirst

[filters]
add = ["-build/include_alpha"]       # 追加的过滤器

[extensions]
headers = ["h", "hh", "hpp", "hxx", "h++", "cuh"]
sources = ["c", "cc", "cpp", "cxx", "c++", "cu"]

[exclude]
files = ["build/**", "third_party/**"]

[fix]
trailing_whitespace = true           # auto-fix 行尾空白
utf8_bom = true                      # auto-fix UTF-8 BOM
crlf = true                          # auto-fix CRLF → LF
block_comments = true                # auto-fix /* */ → //
```

## Rust 结构体

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub verbose: Option<u8>,
    pub line_length: Option<usize>,
    pub output: Option<String>,
    pub recursive: Option<bool>,
    pub quiet: Option<bool>,
    pub repository: Option<String>,
    pub root: Option<String>,
    pub include_order: Option<String>,
    pub filters: Option<FilterConfig>,
    pub extensions: Option<ExtensionConfig>,
    pub exclude: Option<ExcludeConfig>,
    pub fix: Option<FixConfig>,
}

#[derive(Debug, Deserialize)]
pub struct FilterConfig {
    pub add: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExtensionConfig {
    pub headers: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ExcludeConfig {
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FixConfig {
    pub trailing_whitespace: Option<bool>,
    pub utf8_bom: Option<bool>,
    pub crlf: Option<bool>,
    pub block_comments: Option<bool>,
}
```

## 优先级

```
CLI 参数 > cclinter-rs.toml > 内置默认值
```

## 加载函数

```rust
impl Config {
    pub fn load(dir: &Path) -> Result<Option<Self>, LintError>;
    pub fn with_cli_overrides(self, cli: &CliArgs) -> Self;
    pub fn effective_line_length(&self) -> usize {
        self.line_length.unwrap_or(80)
    }
    pub fn effective_verbose(&self) -> u8 {
        self.verbose.unwrap_or(1)
    }
    pub fn effective_output(&self) -> &str {
        self.output.as_deref().unwrap_or("emacs")
    }
}
```

所有 `effective_*()` 方法从 `Option<T>` 字段 fallback 到内置默认值。
