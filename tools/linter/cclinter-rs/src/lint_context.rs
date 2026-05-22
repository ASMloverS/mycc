use std::collections::HashSet;
use std::path::Path;

use crate::cleanse::CleansedLines;
use crate::config::Config;
use crate::error::{ErrorCategory, Violation};
use crate::file_info::FileInfo;
use crate::filter::{FilterSet, NolintSuppression};
use crate::function_state::FunctionState;
use crate::include_state::IncludeState;
use crate::nesting::NestingState;

fn default_header_extensions() -> HashSet<String> {
    ["h", "hh", "hpp", "hxx", "h++", "cuh"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

pub struct LintContext<'a> {
    pub filename: String,
    pub lines: &'a CleansedLines,
    pub nesting: NestingState,
    pub filter: &'a FilterSet,
    pub config: &'a Config,
    pub nolint_suppressions: Vec<NolintSuppression>,
    pub violations: Vec<Violation>,
    pub include_state: IncludeState,
    pub function_state: FunctionState,
    pub file_info: FileInfo,
    pub is_c_file: bool,
    pub is_kernel_file: bool,
}

impl<'a> LintContext<'a> {
    pub fn new(
        filename: &str,
        lines: &'a CleansedLines,
        filter: &'a FilterSet,
        config: &'a Config,
    ) -> Self {
        let exts = default_header_extensions();
        let file_info = FileInfo::new(filename, "", &exts);
        let is_c_file = Path::new(filename)
            .extension()
            .map(|e| e == "c")
            .unwrap_or(false);
        Self {
            filename: filename.to_string(),
            lines,
            nesting: NestingState::new(),
            filter,
            config,
            nolint_suppressions: Vec::new(),
            violations: Vec::new(),
            include_state: IncludeState::new(),
            function_state: FunctionState::new(),
            file_info,
            is_c_file,
            is_kernel_file: false,
        }
    }

    pub fn report(&mut self, linenum: usize, category: ErrorCategory, confidence: u8, message: &str) {
        if self.is_suppressed(linenum, category) {
            return;
        }
        if !self.filter.should_print(category, confidence) {
            return;
        }
        self.violations.push(Violation {
            filename: self.filename.clone(),
            linenum,
            category,
            confidence,
            message: message.to_string(),
        });
    }

    pub fn is_suppressed(&self, linenum: usize, category: ErrorCategory) -> bool {
        self.nolint_suppressions
            .iter()
            .any(|sup| sup.linenum == linenum && (sup.category.is_none() || sup.category == Some(category)))
    }

    pub fn into_violations(self) -> Vec<Violation> {
        self.violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx<'a>(lines: &'a CleansedLines, filter: &'a FilterSet, config: &'a Config) -> LintContext<'a> {
        LintContext::new("test.cc", lines, filter, config)
    }

    #[test]
    fn test_report_violation() {
        let lines = CleansedLines::from_source("int x = 1;");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = make_ctx(&lines, &filter, &config);
        ctx.report(1, ErrorCategory::WhitespaceTab, 5, "Tab found");
        assert_eq!(ctx.violations.len(), 1);
        assert_eq!(ctx.violations[0].linenum, 1);
        assert_eq!(ctx.violations[0].category, ErrorCategory::WhitespaceTab);
    }

    #[test]
    fn test_report_filtered_out() {
        let lines = CleansedLines::from_source("int x = 1;");
        let mut filter = FilterSet::new();
        filter.add("-whitespace");
        let config = Config::default();
        let mut ctx = make_ctx(&lines, &filter, &config);
        ctx.report(1, ErrorCategory::WhitespaceTab, 5, "Tab found");
        assert!(ctx.violations.is_empty());
    }

    #[test]
    fn test_report_nolint_suppressed() {
        let lines = CleansedLines::from_source("int x = 1;");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = make_ctx(&lines, &filter, &config);
        ctx.nolint_suppressions.push(NolintSuppression {
            category: None,
            linenum: 1,
        });
        ctx.report(1, ErrorCategory::WhitespaceTab, 5, "Tab found");
        assert!(ctx.violations.is_empty());
    }

    #[test]
    fn test_into_violations() {
        let lines = CleansedLines::from_source("int x = 1;");
        let filter = FilterSet::new();
        let config = Config::default();
        let mut ctx = make_ctx(&lines, &filter, &config);
        ctx.report(1, ErrorCategory::WhitespaceTab, 5, "Tab found");
        let violations = ctx.into_violations();
        assert_eq!(violations.len(), 1);
    }
}
