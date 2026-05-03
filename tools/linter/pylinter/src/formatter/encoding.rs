use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_encoding(
    source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = source.content.as_str();
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let content = content.replace("\r\n", "\n").replace('\r', "\n");
    let had_newline = content.ends_with('\n');
    let lines: Vec<String> = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::common::source::SourceFile;
    use crate::config::FormatConfig;

    #[test]
    fn strips_bom() {
        let mut src =
            SourceFile::from_string("\u{feff}x = 1\n", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "x = 1\n");
    }

    #[test]
    fn converts_crlf_to_lf() {
        let mut src =
            SourceFile::from_string("x = 1\r\ny = 2\r\n", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "x = 1\ny = 2\n");
    }

    #[test]
    fn converts_cr_to_lf() {
        let mut src =
            SourceFile::from_string("x = 1\ry = 2\r", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "x = 1\ny = 2\n");
    }

    #[test]
    fn preserves_trailing_newline() {
        let mut src = SourceFile::from_string("x = 1\n", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert!(src.content.ends_with('\n'));
    }

    #[test]
    fn preserves_no_trailing_newline() {
        let mut src = SourceFile::from_string("x = 1", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert!(!src.content.ends_with('\n'));
    }

    #[test]
    fn strips_trailing_whitespace() {
        let mut src =
            SourceFile::from_string("x = 1  \ny = 2\t\n", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "x = 1\ny = 2\n");
    }

    #[test]
    fn empty_input_stays_empty() {
        let mut src = SourceFile::from_string("", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "");
    }

    #[test]
    fn strips_bom_and_converts_crlf() {
        let mut src = SourceFile::from_string(
            "\u{feff}x = 1\r\ny = 2\r\n",
            PathBuf::from("test.py"),
        );
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "x = 1\ny = 2\n");
    }

    #[test]
    fn strips_bom_only() {
        let mut src = SourceFile::from_string("\u{feff}", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "");
    }

    #[test]
    fn preserves_multiple_blank_lines() {
        let mut src = SourceFile::from_string("x = 1\n\n\ny = 2\n", PathBuf::from("test.py"));
        super::fix_encoding(&mut src, &FormatConfig::default()).unwrap();
        assert_eq!(src.content, "x = 1\n\n\ny = 2\n");
    }
}
