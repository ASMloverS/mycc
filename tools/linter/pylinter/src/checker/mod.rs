pub mod complexity;
pub mod magic_number;
pub mod naming;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::CheckConfig;

pub fn check_source(source: &SourceFile, config: &CheckConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    diags.extend(naming::check_naming(source, &config.naming));
    diags.extend(complexity::check_complexity(source, &config.complexity));
    diags.extend(magic_number::check_magic_number(source, &config.magic_number));
    diags
}
