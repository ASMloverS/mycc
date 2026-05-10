use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::{AnalysisConfig, AnalysisLevel};

mod basic;

pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel, config: &AnalysisConfig) -> Vec<Diagnostic> {
    match level {
        AnalysisLevel::None => vec![],
        AnalysisLevel::Basic => basic::check(source, config),
        AnalysisLevel::Strict | AnalysisLevel::Deep => basic::check(source, config),
    }
}
