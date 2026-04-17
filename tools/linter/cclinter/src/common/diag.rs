use std::fmt;

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
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sev = match &self.severity {
            Severity::Note => "note",
            Severity::Warning => "warning",
            Severity::Error => "error",
        };
        write!(
            f,
            "{}:{}:{}: {}: {} [{}]",
            self.file, self.line, self.col, sev, self.message, self.rule_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_format() {
        let d = Diagnostic {
            file: "main.c".into(),
            line: 10,
            col: 5,
            severity: Severity::Warning,
            rule_id: "naming-function".into(),
            message: "function should use snake_case".into(),
        };
        assert_eq!(
            format!("{d}"),
            "main.c:10:5: warning: function should use snake_case [naming-function]"
        );
    }
}
