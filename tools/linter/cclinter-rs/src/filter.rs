use crate::error::ErrorCategory;

const LEGACY_CATEGORIES: &[&str] = &["build/class", "readability/streams", "readability/function"];

pub struct FilterSet {
    filters: Vec<String>,
    verbose_level: u8,
}

impl Default for FilterSet {
    fn default() -> Self {
        Self {
            filters: vec!["-build/include_alpha".into()],
            verbose_level: 1,
        }
    }
}

impl FilterSet {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            verbose_level: 1,
        }
    }

    pub fn add(&mut self, filter: &str) {
        self.filters.push(filter.trim().to_string());
    }

    pub fn add_many(&mut self, filters: &str) {
        for f in filters.split(',') {
            let trimmed = f.trim();
            if !trimmed.is_empty() {
                self.add(trimmed);
            }
        }
    }

    pub fn should_print(&self, category: ErrorCategory, confidence: u8) -> bool {
        if confidence < self.verbose_level {
            return false;
        }
        let cat_name = category.name();
        let mut filtered = false;
        for f in &self.filters {
            if let Some(prefix) = f.strip_prefix('-') {
                if cat_name.starts_with(prefix) {
                    filtered = true;
                }
            } else if let Some(prefix) = f.strip_prefix('+') {
                if cat_name.starts_with(prefix) {
                    filtered = false;
                }
            }
        }
        !filtered
    }

    pub fn backup(&self) -> Self {
        self.clone()
    }

    pub fn restore(&mut self, backup: Self) {
        *self = backup;
    }

    pub fn set_verbose(&mut self, level: u8) {
        self.verbose_level = level;
    }
}

impl Clone for FilterSet {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
            verbose_level: self.verbose_level,
        }
    }
}

pub struct NolintSuppression {
    pub category: Option<ErrorCategory>,
    pub linenum: usize,
}

pub fn parse_nolint(raw_line: &str, linenum: usize) -> Vec<NolintSuppression> {
    let mut in_string = false;
    let mut string_char = ' ';
    let mut clean = String::with_capacity(raw_line.len());
    for ch in raw_line.chars() {
        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
            continue;
        }
        clean.push(ch);
    }

    if !clean.contains("NOLINT") {
        return Vec::new();
    }

    let mut results = Vec::new();
    let bytes = clean.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            let comment = &clean[i + 2..];
            let trimmed = comment.trim();
            let target_line;
            let rest;
            if let Some(after) = trimmed.strip_prefix("NOLINTNEXTLINE") {
                target_line = linenum + 1;
                rest = after.trim();
            } else if let Some(after) = trimmed.strip_prefix("NOLINT") {
                target_line = linenum;
                rest = after.trim();
            } else {
                i += 1;
                continue;
            }
            if rest.is_empty() {
                results.push(NolintSuppression {
                    category: None,
                    linenum: target_line,
                });
            } else if rest.starts_with('(') {
                let inner = rest.strip_prefix('(').unwrap_or(rest);
                let inner = inner.strip_suffix(')').unwrap_or(inner);
                let inner = inner.trim();
                if inner == "*" || inner.is_empty() || LEGACY_CATEGORIES.contains(&inner) {
                    results.push(NolintSuppression {
                        category: None,
                        linenum: target_line,
                    });
                } else {
                    match inner.parse::<ErrorCategory>() {
                        Ok(cat) => {
                            results.push(NolintSuppression {
                                category: Some(cat),
                                linenum: target_line,
                            });
                        }
                        Err(_) => {
                            results.push(NolintSuppression {
                                category: None,
                                linenum: target_line,
                            });
                        }
                    }
                }
            }
            break;
        }
        i += 1;
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter() {
        let fs = FilterSet::default();
        assert!(!fs.should_print(ErrorCategory::BuildIncludeAlpha, 1));
        assert!(fs.should_print(ErrorCategory::BuildInclude, 1));
    }

    #[test]
    fn test_negative_filter() {
        let mut fs = FilterSet::new();
        fs.add("-whitespace");
        assert!(!fs.should_print(ErrorCategory::WhitespaceTab, 1));
    }

    #[test]
    fn test_positive_override() {
        let mut fs = FilterSet::new();
        fs.add("-whitespace");
        fs.add("+whitespace/tab");
        assert!(fs.should_print(ErrorCategory::WhitespaceTab, 1));
        assert!(!fs.should_print(ErrorCategory::WhitespaceBraces, 1));
    }

    #[test]
    fn test_add_many() {
        let mut fs = FilterSet::new();
        fs.add_many("-whitespace, +build/include");
        assert!(!fs.should_print(ErrorCategory::WhitespaceTab, 1));
        assert!(fs.should_print(ErrorCategory::BuildInclude, 1));
        assert!(fs.should_print(ErrorCategory::BuildIncludeOrder, 1));
    }

    #[test]
    fn test_confidence_filtering() {
        let mut fs = FilterSet::new();
        fs.set_verbose(3);
        assert!(!fs.should_print(ErrorCategory::BuildInclude, 2));
        assert!(fs.should_print(ErrorCategory::BuildInclude, 3));
    }

    #[test]
    fn test_backup_restore() {
        let mut fs = FilterSet::new();
        fs.add("-whitespace");
        let backup = fs.backup();
        fs.add("-build");
        assert!(!fs.should_print(ErrorCategory::BuildInclude, 1));
        fs.restore(backup);
        assert!(fs.should_print(ErrorCategory::BuildInclude, 1));
        assert!(!fs.should_print(ErrorCategory::WhitespaceTab, 1));
    }

    #[test]
    fn test_prefix_filter() {
        let mut fs = FilterSet::new();
        fs.add("-build/include");
        assert!(!fs.should_print(ErrorCategory::BuildInclude, 1));
        assert!(!fs.should_print(ErrorCategory::BuildIncludeOrder, 1));
        assert!(fs.should_print(ErrorCategory::BuildCpp11, 1));
    }

    #[test]
    fn test_nolint_parse_suppress_all() {
        let results = parse_nolint("int x = 0;  // NOLINT", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, None);
        assert_eq!(results[0].linenum, 5);
    }

    #[test]
    fn test_nolint_parse_category() {
        let results = parse_nolint("int x = 0;  // NOLINT(whitespace/tab)", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, Some(ErrorCategory::WhitespaceTab));
        assert_eq!(results[0].linenum, 5);
    }

    #[test]
    fn test_nolint_parse_unknown_category() {
        let results = parse_nolint("int x = 0;  // NOLINT(unknown/cat)", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, None);
        assert_eq!(results[0].linenum, 5);
    }

    #[test]
    fn test_nolint_legacy_category() {
        let results = parse_nolint("int x = 0;  // NOLINT(build/class)", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, None);
        assert_eq!(results[0].linenum, 5);

        let results = parse_nolint("int x = 0;  // NOLINT(readability/streams)", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, None);
        assert_eq!(results[0].linenum, 10);

        let results = parse_nolint("int x = 0;  // NOLINT(readability/function)", 15);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].category, None);
        assert_eq!(results[0].linenum, 15);
    }
}
