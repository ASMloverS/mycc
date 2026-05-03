use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::CheckConfig;

pub fn check_source(_source: &SourceFile, _config: &CheckConfig) -> Vec<Diagnostic> {
    Vec::new()
}
