use crate::error::{ErrorCategory, Violation};

pub struct FunctionState {
    pub in_function: bool,
    pub function_name: String,
    pub function_start: usize,
    pub function_lines: usize,
    pub is_test_function: bool,
}

impl Default for FunctionState {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionState {
    pub fn new() -> Self {
        Self {
            in_function: false,
            function_name: String::new(),
            function_start: 0,
            function_lines: 0,
            is_test_function: false,
        }
    }

    pub fn begin(&mut self, name: &str, linenum: usize, is_test: bool) {
        self.in_function = true;
        self.function_name = name.to_string();
        self.function_start = linenum;
        self.function_lines = 0;
        self.is_test_function = is_test;
    }

    pub fn end(&mut self) -> Option<Violation> {
        if !self.in_function {
            return None;
        }

        let limit = if self.is_test_function { 400 } else { 250 };
        let over = self.function_lines > limit;

        let result = if over {
            Some(Violation {
                filename: String::new(),
                linenum: self.function_start,
                category: ErrorCategory::ReadabilityFnSize,
                confidence: 100,
                message: format!(
                    "{} has {} lines (limit {})",
                    self.function_name, self.function_lines, limit
                ),
            })
        } else {
            None
        };

        self.in_function = false;
        self.function_name = String::new();
        self.function_start = 0;
        self.function_lines = 0;
        self.is_test_function = false;

        result
    }

    pub fn count_line(&mut self) {
        if self.in_function {
            self.function_lines += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = FunctionState::new();
        assert!(!state.in_function);
    }

    #[test]
    fn test_begin_end_normal() {
        let mut state = FunctionState::new();
        state.begin("foo", 1, false);
        for _ in 0..100 {
            state.count_line();
        }
        assert!(state.end().is_none());
    }

    #[test]
    fn test_function_too_long() {
        let mut state = FunctionState::new();
        state.begin("long_fn", 1, false);
        for _ in 0..251 {
            state.count_line();
        }
        let v = state.end();
        assert!(v.is_some());
        assert_eq!(v.unwrap().category, ErrorCategory::ReadabilityFnSize);
    }

    #[test]
    fn test_test_function_higher_limit() {
        let mut state = FunctionState::new();
        state.begin("test_fn", 1, true);
        for _ in 0..400 {
            state.count_line();
        }
        assert!(state.end().is_none());
    }

    #[test]
    fn test_test_function_over_limit() {
        let mut state = FunctionState::new();
        state.begin("test_fn", 1, true);
        for _ in 0..401 {
            state.count_line();
        }
        let v = state.end();
        assert!(v.is_some());
        assert_eq!(v.unwrap().category, ErrorCategory::ReadabilityFnSize);
    }
}
