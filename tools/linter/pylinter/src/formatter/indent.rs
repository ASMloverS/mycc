use crate::config::FormatConfig;
use crate::cst::CSTSource;

pub fn fix_indent(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    for line in &mut source.lines {
        if line.is_blank {
            continue;
        }
        let indent_str = if config.use_tabs {
            "\t".repeat(line.indent.level)
        } else {
            " ".repeat(line.indent.level * config.indent_width)
        };
        line.indent.raw = indent_str;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::FormatConfig;
    use crate::cst::CSTSource;

    fn config_with(indent_width: usize, use_tabs: bool) -> FormatConfig {
        let mut c = FormatConfig::default();
        c.indent_width = indent_width;
        c.use_tabs = use_tabs;
        c
    }

    #[test]
    fn normalize_2space_to_4space() {
        let mut cst = CSTSource::parse("if True:\n  pass\n").unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "if True:\n    pass\n");
    }

    #[test]
    fn preserve_4space_indent() {
        let input = "if True:\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn nested_indent() {
        let mut cst = CSTSource::parse("if True:\n  if True:\n    pass\n").unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "if True:\n    if True:\n        pass\n");
    }

    #[test]
    fn tab_to_spaces() {
        let mut cst = CSTSource::parse("if True:\n\tpass\n").unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "if True:\n    pass\n");
    }

    #[test]
    fn spaces_to_tabs() {
        let mut cst = CSTSource::parse("if True:\n    pass\n").unwrap();
        super::fix_indent(&mut cst, &config_with(4, true)).unwrap();
        assert_eq!(cst.regenerate(), "if True:\n\tpass\n");
    }

    #[test]
    fn indent_width_2() {
        let mut cst = CSTSource::parse("if True:\n    pass\n").unwrap();
        super::fix_indent(&mut cst, &config_with(2, false)).unwrap();
        assert_eq!(cst.regenerate(), "if True:\n  pass\n");
    }

    #[test]
    fn dedent_correct() {
        let input = "if True:\n    x = 1\ny = 2\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn comment_only_line_indent() {
        let input = "if True:\n    # comment\n    pass\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn comment_only_line_wrong_indent_fixed() {
        let mut cst = CSTSource::parse("if True:\n  # comment\n    pass\n").unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "if True:\n    # comment\n    pass\n");
    }

    #[test]
    fn blank_lines_skipped() {
        let input = "if True:\n    x = 1\n\n    y = 2\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }

    #[test]
    fn mixed_nesting() {
        let input = "def f():\n    if True:\n        pass\n    return\n";
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_indent(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), input);
    }
}
