use std::collections::HashMap;
use std::str::FromStr;

use crate::error::Violation;

pub fn format_emacs(v: &Violation) -> String {
    format!(
        "{}:{}: {} [{}] [{}]",
        v.filename, v.linenum, v.message, v.category, v.confidence
    )
}

pub fn format_vs7(v: &Violation) -> String {
    format!(
        "{}({}): error cpplint: [{}] {} [{}]",
        v.filename, v.linenum, v.category, v.message, v.confidence
    )
}

pub fn format_eclipse(v: &Violation) -> String {
    format!(
        "{}:{}: warning: {}  [{}] [{}]",
        v.filename, v.linenum, v.message, v.category, v.confidence
    )
}

pub fn format_junit(violations: &[Violation]) -> String {
    let mut groups: HashMap<&str, Vec<&Violation>> = HashMap::new();
    for v in violations {
        groups.entry(&v.filename).or_default().push(v);
    }

    let count = violations.len();

    let mut testcases = String::new();
    for (filename, viols) in &groups {
        let mut failures = String::new();
        for v in viols {
            failures.push_str(&format!(
                "{}:{}: {} [{}] [{}]\n",
                v.filename, v.linenum, v.message, v.category, v.confidence
            ));
        }
        testcases.push_str(&format!(
            "<testcase name=\"{}\">\n<failure>\n{}</failure>\n</testcase>\n",
            filename, failures
        ));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<testsuite errors=\"0\" failures=\"{}\" name=\"cpplint\" tests=\"{}\">\n{}</testsuite>",
        count,
        groups.len(),
        testcases
    )
}

pub fn format_sed(v: &Violation, bsd_mode: bool) -> String {
    let expr = if v.message.contains("Tab found") {
        r"s/\t/  /g"
    } else if v.message.contains("Line ends in whitespace") {
        r"s/\s*$//"
    } else {
        return format!(
            "# {}: Cannot auto-fix: {} [{}]",
            v.linenum, v.message, v.category
        );
    };

    if bsd_mode {
        format!("sed -i '' '{}' {}", expr, v.filename)
    } else {
        format!("sed -i '{}' {}", expr, v.filename)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Emacs,
    Vs7,
    Eclipse,
    Junit,
    Sed,
    Gsed,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "emacs" => Ok(OutputFormat::Emacs),
            "vs7" => Ok(OutputFormat::Vs7),
            "eclipse" => Ok(OutputFormat::Eclipse),
            "junit" => Ok(OutputFormat::Junit),
            "sed" => Ok(OutputFormat::Sed),
            "gsed" => Ok(OutputFormat::Gsed),
            _ => Err(format!("unknown output format: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;

    fn make_violation() -> Violation {
        Violation {
            filename: "src/main.cc".to_string(),
            linenum: 42,
            category: ErrorCategory::WhitespaceTab,
            confidence: 5,
            message: "Tab found; replace by spaces".to_string(),
        }
    }

    #[test]
    fn test_emacs_format() {
        let v = make_violation();
        assert_eq!(
            format_emacs(&v),
            "src/main.cc:42: Tab found; replace by spaces [whitespace/tab] [5]"
        );
    }

    #[test]
    fn test_vs7_format() {
        let v = make_violation();
        assert_eq!(
            format_vs7(&v),
            "src/main.cc(42): error cpplint: [whitespace/tab] Tab found; replace by spaces [5]"
        );
    }

    #[test]
    fn test_eclipse_format() {
        let v = make_violation();
        assert_eq!(
            format_eclipse(&v),
            "src/main.cc:42: warning: Tab found; replace by spaces  [whitespace/tab] [5]"
        );
    }

    #[test]
    fn test_junit_format() {
        let v = make_violation();
        let xml = format_junit(&[v]);
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>"));
        assert!(xml.contains("errors=\"0\""));
        assert!(xml.contains("failures=\"1\""));
        assert!(xml.contains("name=\"cpplint\""));
        assert!(xml.contains("<testsuite"));
        assert!(xml.contains("<testcase"));
        assert!(xml.contains("<failure>"));
    }

    #[test]
    fn test_sed_format() {
        let v = make_violation();
        let out = format_sed(&v, false);
        assert!(out.contains("sed -i"));
        assert!(out.contains("src/main.cc"));
    }

    #[test]
    fn test_gsed_format() {
        let v = make_violation();
        let out = format_sed(&v, true);
        assert!(out.contains("sed -i ''"));
        assert!(out.contains("src/main.cc"));
    }
}
