use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;

pub fn check(_source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    vec![]
}
