use std::fmt;
use colored::Colorize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Severity {
    Note,
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
    pub source_line: Option<String>,
}

impl Diagnostic {
    pub fn new(
        file: String,
        line: usize,
        col: usize,
        severity: Severity,
        rule_id: &str,
        message: &str,
    ) -> Self {
        Self {
            file,
            line,
            col,
            severity,
            rule_id: rule_id.to_string(),
            message: message.to_string(),
            source_line: None,
        }
    }

    pub fn new_with_source(
        file: String,
        line: usize,
        col: usize,
        severity: Severity,
        rule_id: &str,
        message: &str,
        source: &str,
    ) -> Self {
        Self {
            file,
            line,
            col,
            severity,
            rule_id: rule_id.to_string(),
            message: message.to_string(),
            source_line: Some(source.to_string()),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev_colored = match &self.severity {
            Severity::Note => "note".normal(),
            Severity::Warning => "warning".yellow(),
            Severity::Error => "error".red(),
        };
        write!(
            f,
            "{}:{}:{}: {}: {} [{}]",
            self.file, self.line, self.col, sev_colored, self.message, self.rule_id
        )?;
        if let Some(ref src) = self.source_line {
            write!(f, "\n  {src}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_format() {
        colored::control::set_override(false);
        let d = Diagnostic {
            file: "main.c".into(),
            line: 10,
            col: 5,
            severity: Severity::Warning,
            rule_id: "naming-function".into(),
            message: "function should use snake_case".into(),
            source_line: None,
        };
        assert_eq!(
            format!("{d}"),
            "main.c:10:5: warning: function should use snake_case [naming-function]"
        );
    }
}
