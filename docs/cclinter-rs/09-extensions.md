# 09 - 扩展检查 + Auto-fix 引擎

## 新增检查 (extensions/*)

### extensions/block_comment

**检测**: 文件原始行中出现 `/*` 或 `*/` 时报错。

```rust
fn check_block_comments(raw_line: &str, linenum: usize, ctx: &mut LintContext) {
    if raw_line.contains("/*") || raw_line.contains("*/") {
        ctx.report(linenum, ErrorCategory::ExtensionsBlockComment, 5,
            "Block comments (/* */) are not allowed; use // comments only");
    }
}
```

**注意**: 此检查在 `CleansedLines` 预处理之后，使用 `raw_lines`（原始行）检测，因为注释已被预处理去除。不使用 `lines`（去注释后）否则 `/* */` 永远不可见。

### extensions/utf8_bom

**检测**: 文件以 UTF-8 BOM (`EF BB BF`) 开头。在原始字节级别检测。

```rust
fn check_utf8_bom(content: &[u8], ctx: &mut LintContext) {
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        ctx.report(1, ErrorCategory::ExtensionsUtf8Bom, 5,
            "UTF-8 BOM detected; remove the BOM bytes");
    }
}
```

### extensions/utf8_invalid

**检测**: 文件包含非 UTF-8 字节序列。在原始字节级别检测。

```rust
fn check_utf8_valid(content: &[u8], ctx: &mut LintContext) {
    match std::str::from_utf8(content) {
        Ok(_) => {}
        Err(e) => {
            let linenum = count_newlines_until(&content[..e.valid_up_to()]) + 1;
            ctx.report(linenum, ErrorCategory::ExtensionsUtf8Invalid, 5,
                "Non-UTF-8 byte sequence detected");
        }
    }
}
```

### extensions/crlf

**检测**: 文件包含 CRLF 换行 (`\r\n`)。在原始字节级别检测，避免依赖 UTF-8 解码。

```rust
fn check_crlf(content: &[u8], ctx: &mut LintContext) {
    if content.windows(2).any(|w| w == b"\r\n") {
        ctx.report(1, ErrorCategory::ExtensionsCrlf, 5,
            "CRLF line endings detected; use LF (\\n) only");
    }
}
```

**说明**: 所有字节级检测（BOM、UTF-8 有效性、CRLF）均在 `ProcessFile` 的原始字节阶段执行，先于 UTF-8 解码。若文件非有效 UTF-8，仅 `extensions/utf8_invalid` 报错，后续 lint 流程跳过该文件。

## Auto-fix 引擎

### 触发条件

`--fix` CLI 参数或配置文件 `[fix]` 节中的选项启用。

### FixEngine

```rust
pub struct FixEngine {
    pub fix_trailing_ws: bool,
    pub fix_utf8_bom: bool,
    pub fix_crlf: bool,
    pub fix_block_comments: bool,
}

pub struct FixReport {
    pub path: PathBuf,
    pub fixed_trailing_ws: usize,
    pub fixed_utf8_bom: bool,
    pub fixed_crlf: usize,
    pub fixed_block_comments: usize,
    pub changed: bool,
}
```

### 修复逻辑

#### 1. 行尾空白符

```rust
fn fix_trailing_whitespace(lines: &mut Vec<String>) -> usize {
    let mut count = 0;
    for line in lines.iter_mut() {
        let trimmed = line.trim_end();
        if trimmed.len() != line.len() {
            *line = trimmed.to_string();
            count += 1;
        }
    }
    count
}
```

#### 2. UTF-8 BOM

```rust
fn fix_utf8_bom(content: &[u8]) -> Option<Vec<u8>> {
    if content.starts_with(&[0xEF, 0xBB, 0xBF]) {
        Some(content[3..].to_vec())
    } else {
        None
    }
}
```

#### 3. CRLF → LF

```rust
fn fix_crlf(content: &[u8]) -> (Vec<u8>, usize) {
    let mut result = Vec::with_capacity(content.len());
    let mut count = 0;
    let mut i = 0;
    while i < content.len() {
        if i + 1 < content.len() && content[i] == b'\r' && content[i + 1] == b'\n' {
            result.push(b'\n');
            count += 1;
            i += 2;
        } else {
            result.push(content[i]);
            i += 1;
        }
    }
    (result, count)
}
```

#### 4. 块注释 → 行注释

**设计要点**: fix 在 `CleansedLines` 之外独立运行，因此必须自行处理字符串/字符字面量内的 `/* */`，避免误转换。

**策略**: 利用 `CleansedLines` 的 `lines`（去注释后）层来确定哪些行位置存在真实注释。比较 `raw_lines` 和 `lines` 的差异，仅在 `raw_lines` 有 `/* */` 而 `lines` 已去除对应内容的区域执行转换。

```rust
fn fix_block_comments(content: &str, cleansed_lines: &CleansedLines) -> (String, usize) {
    // 策略:
    // 1. 比较 raw_lines 与 lines 定位真实注释区域
    // 2. 仅对真实注释区域执行 /* → // 转换
    // 3. 同行多块注释: 逐个处理所有 /* ... */ 对
    //
    // 转换规则:
    // /* single line */ → // single line
    // /* multi
    //    line
    //    comment */ → // multi
    //                  // line
    //                  // comment
    //
    // 保留缩进，去除 * 前缀

    let raw = cleansed_lines.raw_lines();
    let clean = cleansed_lines.lines();
    let mut result = String::new();
    let mut count = 0;

    for (i, (raw_line, clean_line)) in raw.iter().zip(clean.iter()).enumerate() {
        if i == 0 { continue; } // 跳过 marker line

        // 若该行的 raw 与 clean 不同，说明有注释被去除
        if raw_line != clean_line && raw_line.contains("/*") {
            // 找到所有 /* ... */ 对并替换为 //
            let fixed = replace_block_comment_in_line(raw_line);
            result.push_str(&fixed);
            result.push('\n');
            count += 1;
        } else {
            result.push_str(raw_line);
            result.push('\n');
        }
    }

    (result, count)
}

/// 同行内逐个替换 /* ... */ → // ...
/// 正确处理多块注释: /* a */ code /* b */ → // a code // b
fn replace_block_comment_in_line(line: &str) -> String {
    let mut result = String::new();
    let mut remaining = line;

    while let Some(start) = remaining.find("/*") {
        result.push_str(&remaining[..start]);

        if let Some(end_rel) = remaining[start + 2..].find("*/") {
            let comment_text = &remaining[start + 2..start + 2 + end_rel];
            let after = &remaining[start + 2 + end_rel + 2..];
            result.push_str("// ");
            result.push_str(comment_text.trim());
            remaining = after;
        } else {
            // 跨行块注释起始 — 仅替换开头部分
            let comment_text = &remaining[start + 2..];
            result.push_str("// ");
            result.push_str(comment_text.trim());
            remaining = "";
        }
    }

    result.push_str(remaining);
    result
}

fn clean_block_comment_line(line: &str, indent: &str) -> String {
    let trimmed = line.trim_start();
    trimmed.strip_prefix("* ").unwrap_or(trimmed).to_string()
}
```

### fix_file 流程

```rust
impl FixEngine {
    pub fn fix_file(&self, path: &Path) -> Result<FixReport> {
        let raw = std::fs::read(path)?;
        let mut content = raw.clone();
        let mut report = FixReport::new(path);

        // 1. UTF-8 BOM (字节级)
        if self.fix_utf8_bom && content.starts_with(&[0xEF, 0xBB, 0xBF]) {
            content = content[3..].to_vec();
            report.fixed_utf8_bom = true;
        }

        // 2. CRLF → LF (字节级)
        if self.fix_crlf {
            let (fixed, count) = fix_crlf(&content);
            content = fixed;
            report.fixed_crlf = count;
        }

        // 3. 块注释 → 行注释 (需要 UTF-8 + CleansedLines)
        if self.fix_block_comments {
            if let Ok(text) = std::str::from_utf8(&content) {
                let cleansed = CleansedLines::from_source(text);
                let (fixed, count) = fix_block_comments(text, &cleansed);
                content = fixed.into_bytes();
                report.fixed_block_comments = count;
            }
        }

        // 4. 行尾空白符
        if self.fix_trailing_ws {
            if let Ok(text) = std::str::from_utf8(&content) {
                let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
                report.fixed_trailing_ws = fix_trailing_whitespace(&mut lines);
                content = lines.join("\n").into_bytes();
                content.push(b'\n');
            }
        }

        report.changed = content != raw;
        if report.changed {
            std::fs::write(path, &content)?;
        }

        Ok(report)
    }
}
```
