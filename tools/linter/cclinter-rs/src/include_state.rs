use crate::error::{ErrorCategory, Violation};

pub struct IncludeState {
    pub section: u8,
    pub last_include: Option<String>,
    pub last_sorted_section: u8,
    pub include_list: Vec<(String, u8)>,
}

impl Default for IncludeState {
    fn default() -> Self {
        Self::new()
    }
}

impl IncludeState {
    pub fn new() -> Self {
        Self {
            section: 0,
            last_include: None,
            last_sorted_section: 0,
            include_list: Vec::new(),
        }
    }

    pub fn reset(&mut self, section: u8) {
        self.section = section;
    }

    pub fn check_include_order(
        &mut self,
        include_path: &str,
        linenum: usize,
        section: u8,
        filename: &str,
    ) -> Vec<Violation> {
        let mut violations = Vec::new();

        if section < self.section {
            violations.push(Violation {
                filename: filename.to_string(),
                linenum,
                category: ErrorCategory::BuildIncludeOrder,
                confidence: 100,
                message: format!(
                    "{} is in wrong order (section {} after {})",
                    include_path, section, self.section
                ),
            });
        } else if section == self.section {
            if let Some(ref last) = self.last_include {
                if include_path < last.as_str() {
                    violations.push(Violation {
                        filename: filename.to_string(),
                        linenum,
                        category: ErrorCategory::BuildIncludeAlpha,
                        confidence: 100,
                        message: format!("{} should come before {}", include_path, last),
                    });
                }
            }
        }

        self.section = section;
        self.last_include = Some(include_path.to_string());
        self.include_list.push((include_path.to_string(), section));

        violations
    }

    pub fn check_alpha(&self, include_path: &str) -> Option<Violation> {
        if let Some(ref last) = self.last_include {
            if include_path < last.as_str() {
                return Some(Violation {
                    filename: String::new(),
                    linenum: 0,
                    category: ErrorCategory::BuildIncludeAlpha,
                    confidence: 100,
                    message: format!("{} should come before {}", include_path, last),
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headers::{LIKELY_MY_HEADER, C_SYS_HEADER};

    #[test]
    fn test_new_state() {
        let state = IncludeState::new();
        assert_eq!(state.section, 0);
        assert!(state.last_include.is_none());
    }

    #[test]
    fn test_ordered_sections() {
        let mut state = IncludeState::new();
        let v1 = state.check_include_order("stdio.h", 1, C_SYS_HEADER, "test.cc");
        assert!(v1.is_empty());
        let v2 = state.check_include_order("foo.h", 2, LIKELY_MY_HEADER, "test.cc");
        assert!(v2.is_empty());
    }

    #[test]
    fn test_disorder_triggers_violation() {
        let mut state = IncludeState::new();
        let v1 = state.check_include_order("foo.h", 1, LIKELY_MY_HEADER, "test.cc");
        assert!(v1.is_empty());
        let v2 = state.check_include_order("stdio.h", 2, C_SYS_HEADER, "test.cc");
        assert!(!v2.is_empty());
        assert_eq!(v2[0].category, ErrorCategory::BuildIncludeOrder);
    }

    #[test]
    fn test_alpha_order() {
        let mut state = IncludeState::new();
        state.check_include_order("a.h", 1, LIKELY_MY_HEADER, "test.cc");
        let v = state.check_include_order("b.h", 2, LIKELY_MY_HEADER, "test.cc");
        assert!(v.is_empty());
    }

    #[test]
    fn test_alpha_violation() {
        let mut state = IncludeState::new();
        state.last_include = Some("b.h".to_string());
        let result = state.check_alpha("a.h");
        assert!(result.is_some());
        assert_eq!(result.unwrap().category, ErrorCategory::BuildIncludeAlpha);
    }
}
