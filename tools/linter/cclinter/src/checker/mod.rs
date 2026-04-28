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
    diags.extend(naming::check_naming(
        source,
        config.naming.function.as_str(),
        "function",
    ));
    diags.extend(naming::check_naming(
        source,
        config.naming.r#macro.as_str(),
        "macro",
    ));
    diags.extend(naming::check_naming(
        source,
        config.naming.variable.as_str(),
        "variable",
    ));
    diags.extend(naming::check_naming(
        source,
        config.naming.r#type.as_str(),
        "type",
    ));
    diags.extend(naming::check_naming(
        source,
        config.naming.constant.as_str(),
        "constant",
    ));
    diags.extend(include_guard::check_include_guard(source, &config.include_guard));
    diags.extend(complexity::check_complexity(source, &config.complexity));
    diags.extend(magic_number::check_magic_number(source, &config.magic_number));
    diags.extend(unused::check_unused(source, &config.unused));
    diags.extend(prohibited::check_prohibited(
        source,
        config.prohibited_functions.use_default,
        &config.prohibited_functions.extra,
        &config.prohibited_functions.remove,
    ));
    diags.extend(forward_decl::check_forward_decl(source));
    diags
}
