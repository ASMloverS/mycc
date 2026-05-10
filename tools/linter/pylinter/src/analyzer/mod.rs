use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::{AnalysisConfig, AnalysisLevel};

mod basic;
mod strict;

pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel, config: &AnalysisConfig) -> Vec<Diagnostic> {
    match level {
        AnalysisLevel::None => vec![],
        AnalysisLevel::Basic => basic::check(source, config),
        AnalysisLevel::Strict | AnalysisLevel::Deep => {
            let mut diags = basic::check(source, config);
            diags.extend(strict::check(source, config));
            diags
        }
    }
}
