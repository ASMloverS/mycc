use crate::config::{FormatConfig, TrailingWhitespace};
use crate::cst::CSTSource;

pub fn fix_trailing_ws(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.trailing_whitespace == TrailingWhitespace::Preserve {
        return Ok(());
    }
    for line in &mut source.lines {
        if line.comment.is_none() {
            line.trailing_ws.clear();
        }
        let trimmed = line.code.trim_end_matches([' ', '\t']);
        if trimmed.len() != line.code.len() {
            line.code = trimmed.to_string();
        }
        if line.is_blank {
            line.indent.raw.clear();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::{FormatConfig, TrailingWhitespace};
    use crate::cst::CSTSource;

    #[test]
    fn strips_spaces() {
        let mut cst = CSTSource::parse("x = 1   \n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "x = 1\n");
    }

    #[test]
    fn strips_tabs() {
        let mut cst = CSTSource::parse("x = 1\t\n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "x = 1\n");
    }

    #[test]
    fn preserves_code_content() {
        let mut cst = CSTSource::parse("x = 1\ny = 2\n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "x = 1\ny = 2\n");
    }

    #[test]
    fn handles_empty_lines() {
        let mut cst = CSTSource::parse("x = 1\n   \ny = 2\n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "x = 1\n\ny = 2\n");
    }

    #[test]
    fn preserves_inline_comment_spacing() {
        let mut cst = CSTSource::parse("x = 1  # inline\n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "x = 1  # inline\n");
    }

    #[test]
    fn preserve_config_skips_fix() {
        let mut config = FormatConfig::default();
        config.trailing_whitespace = TrailingWhitespace::Preserve;
        let mut cst = CSTSource::parse("x = 1   \n").unwrap();
        super::fix_trailing_ws(&mut cst, &config).unwrap();
        assert_eq!(cst.regenerate(), "x = 1   \n");
    }

    #[test]
    fn comment_only_line_preserved() {
        let mut cst = CSTSource::parse("    # standalone\n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "    # standalone\n");
    }

    #[test]
    fn handles_blank_line_tab_whitespace() {
        let mut cst = CSTSource::parse("x = 1\n\t\t\ny = 2\n").unwrap();
        super::fix_trailing_ws(&mut cst, &FormatConfig::default()).unwrap();
        assert_eq!(cst.regenerate(), "x = 1\n\ny = 2\n");
    }
}
