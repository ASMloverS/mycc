use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use regex::Regex;
use std::sync::LazyLock;

static SNAKE_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*$").unwrap());
static UPPER_SNAKE_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][A-Z0-9_]*$").unwrap());
static PASCAL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap());
static CAMEL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap());

static FUNCTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(?:(?:static|extern|const|inline)\s+)*(?:void|int|char|float|double|long|short|unsigned|signed|struct\s+\w+|enum\s+\w+|\w+_t)\s*\*?\s*(\w+)\s*\("
    ).unwrap()
});
static MACRO_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#define\s+(\w+)").unwrap());
static VARIABLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:(?:static|extern|const)\s+)*(?:int|char|float|double|long|void|unsigned|struct\s+\w+|\w+_t)\s+\*?\s*(\w+)\s*[;=]"
    ).unwrap()
});
static TYPE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"typedef\s+.*?\s+(\w+)\s*;").unwrap());
static CONSTANT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bconst\s+\w+\s+\*?\s*(\w+)\s*[;=]").unwrap());

pub fn check_naming(source: &SourceFile, style: &str, kind: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let pattern_re = naming_regex(style);
    let search_re: &LazyLock<Regex> = match kind {
        "function" => &FUNCTION_RE,
        "macro" => &MACRO_RE,
        "variable" => &VARIABLE_RE,
        "type" => &TYPE_RE,
        "constant" => &CONSTANT_RE,
        _ => return diags,
    };

    for (line_num, line) in source.lines().iter().enumerate() {
        for caps in search_re.captures_iter(line) {
            let name = caps[1].to_string();
            if !pattern_re.is_match(&name) {
                diags.push(Diagnostic::new_with_source(
                    source.path.to_string_lossy().to_string(),
                    line_num + 1,
                    1,
                    Severity::Warning,
                    &format!("readability-naming-{}", kind),
                    &format!("{} '{}' does not follow {} convention", kind, name, style),
                    line,
                ));
            }
        }
    }
    diags
}

fn naming_regex(style: &str) -> &'static LazyLock<Regex> {
    match style {
        "snake_case" => &SNAKE_CASE_RE,
        "upper_snake_case" => &UPPER_SNAKE_CASE_RE,
        "pascal_case" => &PASCAL_CASE_RE,
        "camelCase" => &CAMEL_CASE_RE,
        _ => &SNAKE_CASE_RE,
    }
}
