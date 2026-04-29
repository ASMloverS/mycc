pub mod basic;
pub mod deep;
pub mod strict;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::{AnalysisConfig, AnalysisLevel};

pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel, config: &AnalysisConfig) -> Vec<Diagnostic> {
    match level {
        AnalysisLevel::None => vec![],
        AnalysisLevel::Basic => basic::check(source, config),
        AnalysisLevel::Strict => {
            let mut diags = basic::check(source, config);
            diags.extend(strict::check(source, config));
            diags
        }
        AnalysisLevel::Deep => {
            let mut diags = basic::check(source, config);
            diags.extend(strict::check(source, config));
            diags.extend(deep::check(source, config));
            diags
        }
    }
}
