use crate::config::{BinaryOpBreak, FormatConfig};
use crate::cst::CSTSource;

pub fn fix_binary_op(
    source: &mut CSTSource,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.binary_op_line_break == BinaryOpBreak::After {
        return Ok(());
    }

    for i in 0..source.lines.len() {
        if source.lines[i].is_blank {
            continue;
        }

        let code = source.lines[i].code.trim_end();
        if let Some(op) = trailing_binary_op(code) {
            if i + 1 < source.lines.len() && !source.lines[i + 1].is_blank {
                let new_code = code[..code.len() - op.len()].trim_end();
                source.lines[i].code = new_code.to_string();
                let next_code = source.lines[i + 1].code.trim_start();
                source.lines[i + 1].code = format!("{} {}", op, next_code);
            }
        }
    }

    Ok(())
}

fn trailing_binary_op(code: &str) -> Option<&'static str> {
    for op in &["and", "or"] {
        if code.ends_with(op) && code.len() > op.len() {
            let before = code.as_bytes()[code.len() - op.len() - 1];
            if before == b' ' || before == b'\t' {
                return Some(*op);
            }
        }
    }
    for op in &["<<", ">>", "//", "**"] {
        if code.ends_with(op) {
            return Some(*op);
        }
    }
    match code.as_bytes().last()? {
        b'+' => Some("+"),
        b'-' => Some("-"),
        b'*' => Some("*"),
        b'/' => Some("/"),
        b'%' => Some("%"),
        b'&' => Some("&"),
        b'|' => Some("|"),
        b'^' => Some("^"),
        b'@' => Some("@"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{BinaryOpBreak, FormatConfig};
    use crate::cst::CSTSource;

    fn fixed(input: &str) -> String {
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_binary_op(&mut cst, &FormatConfig::default()).unwrap();
        cst.regenerate()
    }

    fn fixed_after(input: &str) -> String {
        let mut config = FormatConfig::default();
        config.binary_op_line_break = BinaryOpBreak::After;
        let mut cst = CSTSource::parse(input).unwrap();
        super::fix_binary_op(&mut cst, &config).unwrap();
        cst.regenerate()
    }

    #[test]
    fn move_op_to_next_line_before_style() {
        assert_eq!(
            fixed("x = (a +\n     b)\n"),
            "x = (a\n     + b)\n"
        );
    }

    #[test]
    fn keep_op_at_line_end_after_style() {
        assert_eq!(
            fixed_after("x = (a +\n     b)\n"),
            "x = (a +\n     b)\n"
        );
    }

    #[test]
    fn and_or_operators() {
        assert_eq!(
            fixed("if (a and\n    b):\n    pass\n"),
            "if (a\n    and b):\n    pass\n"
        );
    }

    #[test]
    fn no_change_if_not_continuation() {
        assert_eq!(fixed("x = 1 + 2\n"), "x = 1 + 2\n");
    }

    #[test]
    fn minus_operator() {
        assert_eq!(
            fixed("result = (a -\n     b)\n"),
            "result = (a\n     - b)\n"
        );
    }

    #[test]
    fn multiply_operator() {
        assert_eq!(
            fixed("result = (a *\n     b)\n"),
            "result = (a\n     * b)\n"
        );
    }

    #[test]
    fn divide_operator() {
        assert_eq!(
            fixed("result = (a /\n     b)\n"),
            "result = (a\n     / b)\n"
        );
    }

    #[test]
    fn or_operator() {
        assert_eq!(
            fixed("if (x or\n    y):\n    pass\n"),
            "if (x\n    or y):\n    pass\n"
        );
    }

    #[test]
    fn multiple_continuations() {
        assert_eq!(
            fixed("x = (a +\n     b +\n     c)\n"),
            "x = (a\n     + b\n     + c)\n"
        );
    }

    #[test]
    fn empty_input() {
        assert_eq!(fixed(""), "");
    }

    #[test]
    fn no_change_without_operator_at_end() {
        assert_eq!(fixed("x = 1\ny = 2\n"), "x = 1\ny = 2\n");
    }

    #[test]
    fn negative_number_not_matched() {
        assert_eq!(fixed("x = -1\n"), "x = -1\n");
    }

    #[test]
    fn word_ending_in_and_not_matched() {
        assert_eq!(fixed("x = grand\n"), "x = grand\n");
    }

    #[test]
    fn word_ending_in_or_not_matched() {
        assert_eq!(fixed("x = valor\n"), "x = valor\n");
    }

    #[test]
    fn floor_div_operator() {
        assert_eq!(
            fixed("result = (a //\n     b)\n"),
            "result = (a\n     // b)\n"
        );
    }

    #[test]
    fn power_operator() {
        assert_eq!(
            fixed("result = (a **\n     b)\n"),
            "result = (a\n     ** b)\n"
        );
    }

    #[test]
    fn indented_continuation() {
        assert_eq!(
            fixed("def f():\n    x = (a +\n         b)\n"),
            "def f():\n    x = (a\n         + b)\n"
        );
    }

    #[test]
    fn operator_before_blank_line_unchanged() {
        assert_eq!(fixed("x = (a +\n\nb)\n"), "x = (a +\n\nb)\n");
    }

    #[test]
    fn modulo_operator() {
        assert_eq!(
            fixed("result = (a %\n     b)\n"),
            "result = (a\n     % b)\n"
        );
    }

    #[test]
    fn bitwise_and_operator() {
        assert_eq!(
            fixed("result = (a &\n     b)\n"),
            "result = (a\n     & b)\n"
        );
    }

    #[test]
    fn bitwise_or_operator() {
        assert_eq!(
            fixed("result = (a |\n     b)\n"),
            "result = (a\n     | b)\n"
        );
    }

    #[test]
    fn bitwise_xor_operator() {
        assert_eq!(
            fixed("result = (a ^\n     b)\n"),
            "result = (a\n     ^ b)\n"
        );
    }

    #[test]
    fn matrix_multiply_operator() {
        assert_eq!(
            fixed("result = (a @\n     b)\n"),
            "result = (a\n     @ b)\n"
        );
    }

    #[test]
    fn left_shift_operator() {
        assert_eq!(
            fixed("result = (a <<\n     b)\n"),
            "result = (a\n     << b)\n"
        );
    }

    #[test]
    fn right_shift_operator() {
        assert_eq!(
            fixed("result = (a >>\n     b)\n"),
            "result = (a\n     >> b)\n"
        );
    }

    #[test]
    fn trailing_comment_preserved() {
        assert_eq!(
            fixed("x = (a +  # comment\n     b)\n"),
            "x = (a  # comment\n     + b)\n"
        );
    }
}
