pub mod basic;
pub mod deep;
pub mod strict;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::AnalysisLevel;

pub fn analyze_source(source: &SourceFile, level: &AnalysisLevel) -> Vec<Diagnostic> {
    match level {
        AnalysisLevel::None => vec![],
        AnalysisLevel::Basic => basic::analyze_basic(source),
        AnalysisLevel::Strict => {
            let mut diags = basic::analyze_basic(source);
            diags.extend(strict::analyze_strict(source));
            diags
        }
        AnalysisLevel::Deep => {
            let mut diags = basic::analyze_basic(source);
            diags.extend(strict::analyze_strict(source));
            diags.extend(deep::analyze_deep(source));
            diags
        }
    }
}
