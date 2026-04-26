pub mod complexity;
pub mod forward_decl;
pub mod include_guard;
pub mod magic_number;
pub mod naming;
pub mod prohibited;
pub mod unused;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::CheckConfig;

pub fn check_source(source: &SourceFile, config: &CheckConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(naming::check_naming(source, "snake_case", "function"));
    diags.extend(naming::check_naming(source, "upper_snake_case", "macro"));
    diags.extend(naming::check_naming(source, "snake_case", "variable"));
    diags.extend(naming::check_naming(source, "pascal_case", "type"));
    diags.extend(naming::check_naming(source, "upper_snake_case", "constant"));
    diags.extend(include_guard::check_include_guard(source, &config.include_guard));
    diags.extend(complexity::check_complexity(source));
    diags.extend(magic_number::check_magic_number(source));
    diags.extend(unused::check_unused(source));
    diags.extend(prohibited::check_prohibited(source));
    diags.extend(forward_decl::check_forward_decl(source));
    diags
}
