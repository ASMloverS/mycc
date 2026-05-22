use std::fmt;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum LintError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("{0}")]
    InvalidArgs(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    BuildCpp11,
    BuildCpp17,
    BuildDeprecated,
    BuildEndifComment,
    BuildExplicitMakePair,
    BuildForwardDecl,
    BuildHeaderGuard,
    BuildInclude,
    BuildIncludeSubdir,
    BuildIncludeAlpha,
    BuildIncludeOrder,
    BuildIncludeWhatYouUse,
    BuildNamespacesHeaders,
    BuildNamespacesLiterals,
    BuildNamespaces,
    BuildPrintfFormat,
    BuildStorageClass,
    LegalCopyright,
    ReadabilityAltTokens,
    ReadabilityBraces,
    ReadabilityCasting,
    ReadabilityCheck,
    ReadabilityConstructors,
    ReadabilityFnSize,
    ReadabilityInheritance,
    ReadabilityMultilineComment,
    ReadabilityMultilineString,
    ReadabilityNamespace,
    ReadabilityNolint,
    ReadabilityNul,
    ReadabilityTodo,
    ReadabilityUtf8,
    RuntimeArrays,
    RuntimeCasting,
    RuntimeExplicit,
    RuntimeInt,
    RuntimeInit,
    RuntimeInvalidIncrement,
    RuntimeMemberStringReferences,
    RuntimeMemset,
    RuntimeOperator,
    RuntimePrintf,
    RuntimePrintfFormat,
    RuntimeReferences,
    RuntimeString,
    RuntimeThreadsafeFn,
    RuntimeVlog,
    WhitespaceBlankLine,
    WhitespaceBraces,
    WhitespaceComma,
    WhitespaceComments,
    WhitespaceEmptyConditionalBody,
    WhitespaceEmptyIfBody,
    WhitespaceEmptyLoopBody,
    WhitespaceEndOfLine,
    WhitespaceEndingNewline,
    WhitespaceForcolon,
    WhitespaceIndent,
    WhitespaceIndentNamespace,
    WhitespaceLineLength,
    WhitespaceNewline,
    WhitespaceOperators,
    WhitespaceParens,
    WhitespaceSemicolon,
    WhitespaceTab,
    WhitespaceTodo,
    ExtensionsBlockComment,
    ExtensionsUtf8Bom,
    ExtensionsUtf8Invalid,
    ExtensionsCrlf,
}

impl ErrorCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::BuildCpp11 => "build/c++11",
            ErrorCategory::BuildCpp17 => "build/c++17",
            ErrorCategory::BuildDeprecated => "build/deprecated",
            ErrorCategory::BuildEndifComment => "build/endif_comment",
            ErrorCategory::BuildExplicitMakePair => "build/explicit_make_pair",
            ErrorCategory::BuildForwardDecl => "build/forward_decl",
            ErrorCategory::BuildHeaderGuard => "build/header_guard",
            ErrorCategory::BuildInclude => "build/include",
            ErrorCategory::BuildIncludeSubdir => "build/include_subdir",
            ErrorCategory::BuildIncludeAlpha => "build/include_alpha",
            ErrorCategory::BuildIncludeOrder => "build/include_order",
            ErrorCategory::BuildIncludeWhatYouUse => "build/include_what_you_use",
            ErrorCategory::BuildNamespacesHeaders => "build/namespaces_headers",
            ErrorCategory::BuildNamespacesLiterals => "build/namespaces_literals",
            ErrorCategory::BuildNamespaces => "build/namespaces",
            ErrorCategory::BuildPrintfFormat => "build/printf_format",
            ErrorCategory::BuildStorageClass => "build/storage_class",
            ErrorCategory::LegalCopyright => "legal/copyright",
            ErrorCategory::ReadabilityAltTokens => "readability/alt_tokens",
            ErrorCategory::ReadabilityBraces => "readability/braces",
            ErrorCategory::ReadabilityCasting => "readability/casting",
            ErrorCategory::ReadabilityCheck => "readability/check",
            ErrorCategory::ReadabilityConstructors => "readability/constructors",
            ErrorCategory::ReadabilityFnSize => "readability/fn_size",
            ErrorCategory::ReadabilityInheritance => "readability/inheritance",
            ErrorCategory::ReadabilityMultilineComment => "readability/multiline_comment",
            ErrorCategory::ReadabilityMultilineString => "readability/multiline_string",
            ErrorCategory::ReadabilityNamespace => "readability/namespace",
            ErrorCategory::ReadabilityNolint => "readability/nolint",
            ErrorCategory::ReadabilityNul => "readability/nul",
            ErrorCategory::ReadabilityTodo => "readability/todo",
            ErrorCategory::ReadabilityUtf8 => "readability/utf8",
            ErrorCategory::RuntimeArrays => "runtime/arrays",
            ErrorCategory::RuntimeCasting => "runtime/casting",
            ErrorCategory::RuntimeExplicit => "runtime/explicit",
            ErrorCategory::RuntimeInt => "runtime/int",
            ErrorCategory::RuntimeInit => "runtime/init",
            ErrorCategory::RuntimeInvalidIncrement => "runtime/invalid_increment",
            ErrorCategory::RuntimeMemberStringReferences => "runtime/member_string_references",
            ErrorCategory::RuntimeMemset => "runtime/memset",
            ErrorCategory::RuntimeOperator => "runtime/operator",
            ErrorCategory::RuntimePrintf => "runtime/printf",
            ErrorCategory::RuntimePrintfFormat => "runtime/printf_format",
            ErrorCategory::RuntimeReferences => "runtime/references",
            ErrorCategory::RuntimeString => "runtime/string",
            ErrorCategory::RuntimeThreadsafeFn => "runtime/threadsafe_fn",
            ErrorCategory::RuntimeVlog => "runtime/vlog",
            ErrorCategory::WhitespaceBlankLine => "whitespace/blank_line",
            ErrorCategory::WhitespaceBraces => "whitespace/braces",
            ErrorCategory::WhitespaceComma => "whitespace/comma",
            ErrorCategory::WhitespaceComments => "whitespace/comments",
            ErrorCategory::WhitespaceEmptyConditionalBody => "whitespace/empty_conditional_body",
            ErrorCategory::WhitespaceEmptyIfBody => "whitespace/empty_if_body",
            ErrorCategory::WhitespaceEmptyLoopBody => "whitespace/empty_loop_body",
            ErrorCategory::WhitespaceEndOfLine => "whitespace/end_of_line",
            ErrorCategory::WhitespaceEndingNewline => "whitespace/ending_newline",
            ErrorCategory::WhitespaceForcolon => "whitespace/forcolon",
            ErrorCategory::WhitespaceIndent => "whitespace/indent",
            ErrorCategory::WhitespaceIndentNamespace => "whitespace/indent_namespace",
            ErrorCategory::WhitespaceLineLength => "whitespace/line_length",
            ErrorCategory::WhitespaceNewline => "whitespace/newline",
            ErrorCategory::WhitespaceOperators => "whitespace/operators",
            ErrorCategory::WhitespaceParens => "whitespace/parens",
            ErrorCategory::WhitespaceSemicolon => "whitespace/semicolon",
            ErrorCategory::WhitespaceTab => "whitespace/tab",
            ErrorCategory::WhitespaceTodo => "whitespace/todo",
            ErrorCategory::ExtensionsBlockComment => "extensions/block_comment",
            ErrorCategory::ExtensionsUtf8Bom => "extensions/utf8_bom",
            ErrorCategory::ExtensionsUtf8Invalid => "extensions/utf8_invalid",
            ErrorCategory::ExtensionsCrlf => "extensions/crlf",
        }
    }

    pub fn group(&self) -> &'static str {
        let name = self.name();
        let idx = name.find('/').unwrap();
        &name[..idx]
    }

    pub fn all() -> &'static [ErrorCategory] {
        &[
            ErrorCategory::BuildCpp11,
            ErrorCategory::BuildCpp17,
            ErrorCategory::BuildDeprecated,
            ErrorCategory::BuildEndifComment,
            ErrorCategory::BuildExplicitMakePair,
            ErrorCategory::BuildForwardDecl,
            ErrorCategory::BuildHeaderGuard,
            ErrorCategory::BuildInclude,
            ErrorCategory::BuildIncludeSubdir,
            ErrorCategory::BuildIncludeAlpha,
            ErrorCategory::BuildIncludeOrder,
            ErrorCategory::BuildIncludeWhatYouUse,
            ErrorCategory::BuildNamespacesHeaders,
            ErrorCategory::BuildNamespacesLiterals,
            ErrorCategory::BuildNamespaces,
            ErrorCategory::BuildPrintfFormat,
            ErrorCategory::BuildStorageClass,
            ErrorCategory::LegalCopyright,
            ErrorCategory::ReadabilityAltTokens,
            ErrorCategory::ReadabilityBraces,
            ErrorCategory::ReadabilityCasting,
            ErrorCategory::ReadabilityCheck,
            ErrorCategory::ReadabilityConstructors,
            ErrorCategory::ReadabilityFnSize,
            ErrorCategory::ReadabilityInheritance,
            ErrorCategory::ReadabilityMultilineComment,
            ErrorCategory::ReadabilityMultilineString,
            ErrorCategory::ReadabilityNamespace,
            ErrorCategory::ReadabilityNolint,
            ErrorCategory::ReadabilityNul,
            ErrorCategory::ReadabilityTodo,
            ErrorCategory::ReadabilityUtf8,
            ErrorCategory::RuntimeArrays,
            ErrorCategory::RuntimeCasting,
            ErrorCategory::RuntimeExplicit,
            ErrorCategory::RuntimeInt,
            ErrorCategory::RuntimeInit,
            ErrorCategory::RuntimeInvalidIncrement,
            ErrorCategory::RuntimeMemberStringReferences,
            ErrorCategory::RuntimeMemset,
            ErrorCategory::RuntimeOperator,
            ErrorCategory::RuntimePrintf,
            ErrorCategory::RuntimePrintfFormat,
            ErrorCategory::RuntimeReferences,
            ErrorCategory::RuntimeString,
            ErrorCategory::RuntimeThreadsafeFn,
            ErrorCategory::RuntimeVlog,
            ErrorCategory::WhitespaceBlankLine,
            ErrorCategory::WhitespaceBraces,
            ErrorCategory::WhitespaceComma,
            ErrorCategory::WhitespaceComments,
            ErrorCategory::WhitespaceEmptyConditionalBody,
            ErrorCategory::WhitespaceEmptyIfBody,
            ErrorCategory::WhitespaceEmptyLoopBody,
            ErrorCategory::WhitespaceEndOfLine,
            ErrorCategory::WhitespaceEndingNewline,
            ErrorCategory::WhitespaceForcolon,
            ErrorCategory::WhitespaceIndent,
            ErrorCategory::WhitespaceIndentNamespace,
            ErrorCategory::WhitespaceLineLength,
            ErrorCategory::WhitespaceNewline,
            ErrorCategory::WhitespaceOperators,
            ErrorCategory::WhitespaceParens,
            ErrorCategory::WhitespaceSemicolon,
            ErrorCategory::WhitespaceTab,
            ErrorCategory::WhitespaceTodo,
            ErrorCategory::ExtensionsBlockComment,
            ErrorCategory::ExtensionsUtf8Bom,
            ErrorCategory::ExtensionsUtf8Invalid,
            ErrorCategory::ExtensionsCrlf,
        ]
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl FromStr for ErrorCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ErrorCategory::all()
            .iter()
            .find(|cat| cat.name() == s)
            .copied()
            .ok_or_else(|| format!("unknown error category: {}", s))
    }
}

#[derive(Debug, Clone)]
pub struct Violation {
    pub filename: String,
    pub linenum: usize,
    pub category: ErrorCategory,
    pub confidence: u8,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_roundtrip() {
        let cat = ErrorCategory::BuildInclude;
        let s = cat.to_string();
        assert_eq!(s, "build/include");
        let parsed: ErrorCategory = s.parse().unwrap();
        assert_eq!(parsed, cat);
    }

    #[test]
    fn test_all_70_categories() {
        let all = ErrorCategory::all();
        assert_eq!(all.len(), 70);
        let mut names = std::collections::HashSet::new();
        for cat in all {
            let name = cat.name().to_string();
            assert!(!names.contains(&name), "duplicate name: {}", name);
            names.insert(name);
        }
        assert_eq!(names.len(), 70);
    }

    #[test]
    fn test_unknown_parse_fails() {
        let result: Result<ErrorCategory, _> = "nonexistent/category".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_group() {
        assert_eq!(ErrorCategory::BuildInclude.group(), "build");
        assert_eq!(ErrorCategory::ExtensionsCrlf.group(), "extensions");
    }

    #[test]
    fn test_group_counts() {
        let all = ErrorCategory::all();
        let build = all.iter().filter(|c| c.group() == "build").count();
        let legal = all.iter().filter(|c| c.group() == "legal").count();
        let readability = all.iter().filter(|c| c.group() == "readability").count();
        let runtime = all.iter().filter(|c| c.group() == "runtime").count();
        let whitespace = all.iter().filter(|c| c.group() == "whitespace").count();
        let extensions = all.iter().filter(|c| c.group() == "extensions").count();
        assert_eq!(build, 17);
        assert_eq!(legal, 1);
        assert_eq!(readability, 14);
        assert_eq!(runtime, 15);
        assert_eq!(whitespace, 19);
        assert_eq!(extensions, 4);
    }

    #[test]
    fn test_violation_fields() {
        let v = Violation {
            filename: "foo.cpp".to_string(),
            linenum: 42,
            category: ErrorCategory::WhitespaceTab,
            confidence: 80,
            message: "Tab found".to_string(),
        };
        assert_eq!(v.filename, "foo.cpp");
        assert_eq!(v.linenum, 42);
        assert_eq!(v.category, ErrorCategory::WhitespaceTab);
        assert_eq!(v.confidence, 80);
        assert_eq!(v.message, "Tab found");
    }
}
