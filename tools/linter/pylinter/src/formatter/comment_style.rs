use crate::config::{CommentStyle, FormatConfig};
use crate::cst::CSTSource;

pub fn fix_comment_style(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.comment_style == CommentStyle::Preserve {
        return Ok(());
    }
    for (i, line) in source.lines.iter_mut().enumerate() {
        if let Some(ref comment) = line.comment {
            if is_protected_comment(comment, i) {
                continue;
            }

            line.comment = Some(normalize_comment(comment));

            if !line.code.is_empty() {
                line.trailing_ws = "  ".to_string();
            }
        }
    }

    Ok(())
}

fn is_protected_comment(comment: &str, line_index: usize) -> bool {
    (line_index == 0 && comment.starts_with("#!"))
        || (line_index <= 1
            && (comment.contains("-*- coding:") || comment.contains("-*- encoding:")))
}

fn normalize_comment(comment: &str) -> String {
    let after_hash = &comment[1..];
    let trimmed = after_hash.trim_start();
    if trimmed.is_empty() {
        return "#".to_string();
    }
    format!("# {}", trimmed)
}

#[cfg(test)]
mod tests {
    use crate::config::{CommentStyle, FormatConfig};
    use crate::cst::CSTSource;

    fn fixed(input: &str) -> String {
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_comment_style(&mut cst, &FormatConfig::default()).unwrap();
        cst.regenerate()
    }

    fn fixed_preserve(input: &str) -> String {
        let mut config = FormatConfig::default();
        config.comment_style = CommentStyle::Preserve;
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_comment_style(&mut cst, &config).unwrap();
        cst.regenerate()
    }

    #[test]
    fn add_space_after_hash() {
        assert_eq!(fixed("#comment\n"), "# comment\n");
    }

    #[test]
    fn preserve_correct_comment() {
        assert_eq!(fixed("# comment\n"), "# comment\n");
    }

    #[test]
    fn inline_comment_spacing() {
        assert_eq!(fixed("x = 1 # inline\n"), "x = 1  # inline\n");
    }

    #[test]
    fn inline_comment_no_space_after_hash() {
        assert_eq!(fixed("x = 1 #inline\n"), "x = 1  # inline\n");
    }

    #[test]
    fn preserve_shebang() {
        assert_eq!(
            fixed("#!/usr/bin/env python3\n"),
            "#!/usr/bin/env python3\n"
        );
    }

    #[test]
    fn preserve_encoding_declaration() {
        assert_eq!(
            fixed("# -*- coding: utf-8 -*-\n"),
            "# -*- coding: utf-8 -*-\n"
        );
    }

    #[test]
    fn hash_in_string_not_treated_as_comment() {
        assert_eq!(
            fixed("x = \"# not a comment\"\n"),
            "x = \"# not a comment\"\n"
        );
    }

    #[test]
    fn empty_comment_stays_hash() {
        assert_eq!(fixed("#\n"), "#\n");
    }

    #[test]
    fn normalize_multiple_spaces_after_hash() {
        assert_eq!(fixed("#  comment\n"), "# comment\n");
    }

    #[test]
    fn inline_already_correct_spacing() {
        assert_eq!(fixed("x = 1  # inline\n"), "x = 1  # inline\n");
    }

    #[test]
    fn preserve_config_skips_fix() {
        assert_eq!(fixed_preserve("#comment\n"), "#comment\n");
    }

    #[test]
    fn encoding_on_second_line_protected() {
        assert_eq!(
            fixed("#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n"),
            "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n"
        );
    }

    #[test]
    fn encoding_beyond_line_two_is_normalized() {
        assert_eq!(
            fixed("x = 1\ny = 2\n#-*- coding: utf-8 -*-\n"),
            "x = 1\ny = 2\n# -*- coding: utf-8 -*-\n"
        );
    }

    #[test]
    fn standalone_comment_with_indent() {
        assert_eq!(
            fixed("def foo():\n    #comment\n    pass\n"),
            "def foo():\n    # comment\n    pass\n"
        );
    }

    #[test]
    fn multiple_comments_in_file() {
        assert_eq!(
            fixed("#comment1\nx = 1 #inline\n#comment2\n"),
            "# comment1\nx = 1  # inline\n# comment2\n"
        );
    }

    #[test]
    fn no_comments_no_change() {
        assert_eq!(fixed("x = 1\ny = 2\n"), "x = 1\ny = 2\n");
    }

    #[test]
    fn shebang_with_code() {
        assert_eq!(
            fixed("#!/usr/bin/env python3\n#comment\nx = 1\n"),
            "#!/usr/bin/env python3\n# comment\nx = 1\n"
        );
    }
}
