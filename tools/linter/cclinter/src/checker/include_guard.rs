use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;

pub fn check_include_guard(_source: &SourceFile) -> Vec<Diagnostic> {
    vec![]
}
