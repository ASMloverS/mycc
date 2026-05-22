# 01 - 错误类别 + Violation 类型

## ErrorCategory enum

66 个原始类别（cpplint 2.0.2）+ 4 个扩展类别 = 70 个。
### build/* (17)

| 变体名 | 类别字符串 |
|--------|-----------|
| BuildCpp11 | build/c++11 |
| BuildCpp17 | build/c++17 |
| BuildDeprecated | build/deprecated |
| BuildEndifComment | build/endif_comment |
| BuildExplicitMakePair | build/explicit_make_pair |
| BuildForwardDecl | build/forward_decl |
| BuildHeaderGuard | build/header_guard |
| BuildInclude | build/include |
| BuildIncludeSubdir | build/include_subdir |
| BuildIncludeAlpha | build/include_alpha |
| BuildIncludeOrder | build/include_order |
| BuildIncludeWhatYouUse | build/include_what_you_use |
| BuildNamespacesHeaders | build/namespaces_headers |
| BuildNamespacesLiterals | build/namespaces_literals |
| BuildNamespaces | build/namespaces |
| BuildPrintfFormat | build/printf_format |
| BuildStorageClass | build/storage_class |

### legal/* (1)

| 变体名 | 类别字符串 |
|--------|-----------|
| LegalCopyright | legal/copyright |

### readability/* (14)

| 变体名 | 类别字符串 |
|--------|-----------|
| ReadabilityAltTokens | readability/alt_tokens |
| ReadabilityBraces | readability/braces |
| ReadabilityCasting | readability/casting |
| ReadabilityCheck | readability/check |
| ReadabilityConstructors | readability/constructors |
| ReadabilityFnSize | readability/fn_size |
| ReadabilityInheritance | readability/inheritance |
| ReadabilityMultilineComment | readability/multiline_comment |
| ReadabilityMultilineString | readability/multiline_string |
| ReadabilityNamespace | readability/namespace |
| ReadabilityNolint | readability/nolint |
| ReadabilityNul | readability/nul |
| ReadabilityTodo | readability/todo |
| ReadabilityUtf8 | readability/utf8 |

### runtime/* (15)

| 变体名 | 类别字符串 |
|--------|-----------|
| RuntimeArrays | runtime/arrays |
| RuntimeCasting | runtime/casting |
| RuntimeExplicit | runtime/explicit |
| RuntimeInt | runtime/int |
| RuntimeInit | runtime/init |
| RuntimeInvalidIncrement | runtime/invalid_increment |
| RuntimeMemberStringReferences | runtime/member_string_references |
| RuntimeMemset | runtime/memset |
| RuntimeOperator | runtime/operator |
| RuntimePrintf | runtime/printf |
| RuntimePrintfFormat | runtime/printf_format |
| RuntimeReferences | runtime/references |
| RuntimeString | runtime/string |
| RuntimeThreadsafeFn | runtime/threadsafe_fn |
| RuntimeVlog | runtime/vlog |

### whitespace/* (19)

| 变体名 | 类别字符串 |
|--------|-----------|
| WhitespaceBlankLine | whitespace/blank_line |
| WhitespaceBraces | whitespace/braces |
| WhitespaceComma | whitespace/comma |
| WhitespaceComments | whitespace/comments |
| WhitespaceEmptyConditionalBody | whitespace/empty_conditional_body |
| WhitespaceEmptyIfBody | whitespace/empty_if_body |
| WhitespaceEmptyLoopBody | whitespace/empty_loop_body |
| WhitespaceEndOfLine | whitespace/end_of_line |
| WhitespaceEndingNewline | whitespace/ending_newline |
| WhitespaceForcolon | whitespace/forcolon |
| WhitespaceIndent | whitespace/indent |
| WhitespaceIndentNamespace | whitespace/indent_namespace |
| WhitespaceLineLength | whitespace/line_length |
| WhitespaceNewline | whitespace/newline |
| WhitespaceOperators | whitespace/operators |
| WhitespaceParens | whitespace/parens |
| WhitespaceSemicolon | whitespace/semicolon |
| WhitespaceTab | whitespace/tab |
| WhitespaceTodo | whitespace/todo |

### extensions/* (4) — 新增

| 变体名 | 类别字符串 |
|--------|-----------|
| ExtensionsBlockComment | extensions/block_comment |
| ExtensionsUtf8Bom | extensions/utf8_bom |
| ExtensionsUtf8Invalid | extensions/utf8_invalid |
| ExtensionsCrlf | extensions/crlf |

## 方法

```rust
impl ErrorCategory {
    fn name(&self) -> &'static str;     // "build/include"
    fn group(&self) -> &'static str;    // "build"
    fn all() -> &'static [ErrorCategory]; // 全部 70 个
}

impl FromStr for ErrorCategory { /* "build/include" → ErrorCategory::BuildInclude */ }
impl Display for ErrorCategory { /* 同 name() */ }
```

## Violation struct

```rust
pub struct Violation {
    pub filename: String,
    pub linenum: usize,      // 1-indexed
    pub category: ErrorCategory,
    pub confidence: u8,      // 1-5, 5 = certain
    pub message: String,
}
```

## 置信度语义

| 级别 | 含义 |
|------|------|
| 1 | 低置信度，可能是合法构造 |
| 2-3 | 中等 |
| 4 | 较高置信度 |
| 5 | 确定是问题 |

当 `confidence < verbose_level` 时，该 violation 被过滤不输出。默认 `verbose_level = 1`（即所有 confidence ≥ 1 的 violation 均输出）。
