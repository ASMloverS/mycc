use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::{AnalysisConfig, AnalysisLevel};

pub fn analyze_source(_source: &SourceFile, _level: &AnalysisLevel, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    Vec::new()
}
