use crate::config::FormatConfig;
use crate::cst::{CSTLine, CSTSource, IndentInfo};

pub fn fix_blank_lines(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if source.lines.is_empty() {
        return Ok(());
    }

    strip_leading_blanks(&mut source.lines);
    source.lines = normalize(&source.lines, config);
    strip_trailing_blanks(&mut source.lines);

    Ok(())
}

/// Blank-line indent level is irrelevant: `trailing_ws` clears `indent.raw`
/// on every blank line downstream, so the synthetic level set here is never emitted.
fn make_blank() -> CSTLine {
    CSTLine {
        num: 0,
        indent: IndentInfo {
            level: 0,
            raw: String::new(),
            width: 0,
            uses_tabs: false,
        },
        tokens: Vec::new(),
        raw_content: "\n".to_string(),
        code: String::new(),
        trailing_ws: String::new(),
        comment: None,
        is_blank: true,
    }
}

fn strip_leading_blanks(lines: &mut Vec<CSTLine>) {
    match lines.iter().position(|l| !l.is_blank) {
        Some(0) => {}
        Some(n) => {
            lines.drain(0..n);
        }
        None => {
            lines.clear();
        }
    }
}

fn strip_trailing_blanks(lines: &mut Vec<CSTLine>) {
    while lines.last().is_some_and(|l| l.is_blank) {
        lines.pop();
    }
}

fn prev_non_blank(result: &[CSTLine]) -> Option<&CSTLine> {
    result.iter().rev().find(|l| !l.is_blank)
}

fn has_prev_at_level(result: &[CSTLine], level: usize) -> bool {
    result.iter().rev().any(|l| !l.is_blank && l.indent.level == level)
}

fn is_decorator(line: &CSTLine) -> bool {
    !line.is_blank && line.code.starts_with('@')
}

fn is_def(line: &CSTLine) -> bool {
    line.code.starts_with("def ") || line.code.starts_with("async def ")
}

fn is_class(line: &CSTLine) -> bool {
    line.code.starts_with("class ")
}

fn required_blanks(result: &[CSTLine], line: &CSTLine, config: &FormatConfig) -> usize {
    if result.is_empty() {
        return 0;
    }

    if let Some(p) = prev_non_blank(result) {
        if p.code.starts_with('@') && p.indent.level == line.indent.level {
            return 0;
        }
    }

    if line.indent.level == 0 {
        if is_class(line) {
            config.blank_lines_before_class
        } else if is_def(line) {
            config.blank_lines_before_function
        } else {
            0
        }
    } else if is_def(line) || is_class(line) {
        if has_prev_at_level(result, line.indent.level) {
            config.blank_lines_inside_class
        } else {
            0
        }
    } else {
        0
    }
}

fn normalize(lines: &[CSTLine], config: &FormatConfig) -> Vec<CSTLine> {
    let mut result: Vec<CSTLine> = Vec::new();

    for i in 0..lines.len() {
        let line = &lines[i];
        if line.is_blank {
            result.push(line.clone());
        } else {
            // If this line is a decorator, peek ahead past blanks and chained
            // decorators to find the decorated def/class so we compute the
            // correct blank-line requirement for the whole decorator group.
            let target = if is_decorator(line) {
                lines[i + 1..]
                    .iter()
                    .find(|l| !l.is_blank && !is_decorator(l))
                    .filter(|l| is_def(l) || is_class(l))
                    .unwrap_or(line)
            } else {
                line
            };

            let blank_count = result.iter().rev().take_while(|l| l.is_blank).count();
            let req = required_blanks(&result, target, config);

            let target_count = if req > 0 {
                req
            } else if result.is_empty() {
                0
            } else {
                blank_count.min(config.max_consecutive_blank_lines)
            };

            for _ in 0..blank_count.saturating_sub(target_count) {
                result.pop();
            }
            for _ in 0..target_count.saturating_sub(blank_count) {
                result.push(make_blank());
            }

            result.push(line.clone());
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::config::FormatConfig;
    use crate::cst::CSTSource;

    #[test]
    fn blank_lines_before_toplevel_function() {
        let mut cst = CSTSource::parse("import os\ndef foo():\n    pass\n").unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(
            cst.regenerate(),
            "import os\n\n\ndef foo():\n    pass\n"
        );
    }

    #[test]
    fn blank_lines_before_toplevel_class() {
        let mut cst = CSTSource::parse("x = 1\nclass Foo:\n    pass\n").unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(
            cst.regenerate(),
            "x = 1\n\n\nclass Foo:\n    pass\n"
        );
    }

    #[test]
    fn blank_lines_inside_class() {
        let input = "class Foo:\n    def bar(self):\n        pass\n    def baz(self):\n        pass\n";
        let expected =
            "class Foo:\n    def bar(self):\n        pass\n\n    def baz(self):\n        pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn collapse_consecutive_blanks() {
        let input = "x = 1\n\n\n\n\ny = 2\n";
        let expected = "x = 1\n\n\ny = 2\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn trim_leading_trailing_blanks() {
        let input = "\n\n\nx = 1\n\n\n";
        let expected = "x = 1\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn no_blanks_before_first_def() {
        let input = "def foo():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn two_toplevel_functions() {
        let input = "def foo():\n    pass\ndef bar():\n    pass\n";
        let expected = "def foo():\n    pass\n\n\ndef bar():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn excess_blanks_collapsed_to_required() {
        let input = "def foo():\n    pass\n\n\n\n\ndef bar():\n    pass\n";
        let expected = "def foo():\n    pass\n\n\ndef bar():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn decorator_not_separated_from_def() {
        let input = "@decorator\ndef foo():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn decorated_toplevel_function_gets_blanks() {
        let input = "import os\n@decorator\ndef foo():\n    pass\n";
        let expected = "import os\n\n\n@decorator\ndef foo():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn decorated_method_inside_class_gets_blanks() {
        let input = "class Foo:\n    x = 1\n    @property\n    def bar(self):\n        return self._bar\n";
        let expected = "class Foo:\n    x = 1\n\n    @property\n    def bar(self):\n        return self._bar\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn stacked_decorators() {
        let input = "import os\n@d1\n@d2\ndef foo():\n    pass\n";
        let expected = "import os\n\n\n@d1\n@d2\ndef foo():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn decorated_toplevel_class() {
        let input = "import os\n@dataclass\nclass Foo:\n    pass\n";
        let expected = "import os\n\n\n@dataclass\nclass Foo:\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn three_methods_in_class() {
        let input = concat!(
            "class Foo:\n",
            "    def a(self):\n",
            "        pass\n",
            "    def b(self):\n",
            "        pass\n",
            "    def c(self):\n",
            "        pass\n",
        );
        let expected = concat!(
            "class Foo:\n",
            "    def a(self):\n",
            "        pass\n",
            "\n",
            "    def b(self):\n",
            "        pass\n",
            "\n",
            "    def c(self):\n",
            "        pass\n",
        );
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn async_def_treated_as_def() {
        let input = "import os\nasync def foo():\n    pass\n";
        let expected = "import os\n\n\nasync def foo():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }

    #[test]
    fn all_blank_input_cleared() {
        let input = "\n\n\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "");
    }

    #[test]
    fn preserve_existing_spacing() {
        let input = "import os\n\n\ndef foo():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn class_followed_by_function() {
        let input = "class Foo:\n    pass\ndef bar():\n    pass\n";
        let expected = "class Foo:\n    pass\n\n\ndef bar():\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_blank_lines(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), expected);
    }
}
