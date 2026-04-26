mod common;

use std::process::Command;

#[test]
fn help_flag_works() {
    let bin = common::get_bin();
    let output = Command::new(&bin).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cclinter"));
    assert!(stdout.contains("--check"));
    assert!(stdout.contains("--diff"));
    assert!(stdout.contains("--in-place"));
    assert!(stdout.contains("--format-only"));
    assert!(stdout.contains("--analysis-level"));
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--exclude"));
    assert!(stdout.contains("--jobs"));
}

use cclinter::common::source::SourceFile;
use cclinter::config::{BraceStyle, FormatConfig};
use cclinter::formatter::blank_lines::fix_blank_lines;
use cclinter::formatter::braces::fix_braces;
use cclinter::formatter::encoding::fix_encoding;
use cclinter::formatter::indent::fix_indent;
use cclinter::formatter::spacing::fix_spacing;

use cclinter::formatter::comments::fix_comments;
use cclinter::config::CommentStyle;

use cclinter::formatter::pointer_style::fix_pointer_style;
use cclinter::config::PointerAlignment;

use std::path::PathBuf;
#[test]
fn test_strip_trailing_whitespace() {
    let mut src = SourceFile::from_string("int x = 1;   \nint y = 2;\t\n", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int x = 1;\nint y = 2;\n");
}

#[test]
fn test_crlf_to_lf() {
    let mut src = SourceFile::from_string("line1\r\nline2\r\n", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "line1\nline2\n");
}

#[test]
fn test_remove_bom() {
    let mut src = SourceFile::from_string("\u{feff}int x;", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int x;");
}

#[test]
fn test_combined_encoding_fixes() {
    let mut src = SourceFile::from_string("\u{feff}int x = 1;   \r\nint y = 2;\t\r\n", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int x = 1;\nint y = 2;\n");
}

#[test]
fn test_empty_input() {
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_bom_only() {
    let mut src = SourceFile::from_string("\u{feff}", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_no_trailing_newline() {
    let mut src = SourceFile::from_string("int x;", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int x;");
}

#[test]
fn test_standalone_cr() {
    let mut src = SourceFile::from_string("line1\rline2\r", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "line1\nline2\n");
}

#[test]
fn test_whitespace_only_line() {
    let mut src = SourceFile::from_string("a\n   \nb\n", PathBuf::from("test.c"));
    fix_encoding(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "a\n\nb\n");
}

#[test]
fn test_tab_to_spaces() {
    let mut src = SourceFile::from_string("int main() {\n\tint x = 1;\n\treturn 0;\n}\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int main() {\n  int x = 1;\n  return 0;\n}\n");
}

#[test]
fn test_nested_indent() {
    let mut src = SourceFile::from_string("void f() {\n\tif (1) {\n\t\treturn;\n\t}\n}\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "void f() {\n  if (1) {\n    return;\n  }\n}\n");
}

#[test]
fn test_no_tabs_unchanged() {
    let mut src = SourceFile::from_string("int x;\n    int y;\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int x;\n    int y;\n");
}

#[test]
fn test_custom_indent_width() {
    let mut config = FormatConfig::default();
    config.indent_width = 4;
    let mut src = SourceFile::from_string("\tint x;\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &config).unwrap();
    assert_eq!(src.content, "    int x;\n");
}

#[test]
fn test_indent_empty_input() {
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_indent_tab_only_line() {
    let mut src = SourceFile::from_string("\t\t\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "    \n");
}

#[test]
fn test_indent_no_trailing_newline() {
    let mut src = SourceFile::from_string("\tint x;", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "  int x;");
}

#[test]
fn test_indent_space_before_tab_unchanged() {
    let mut src = SourceFile::from_string(" \tcode\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, " \tcode\n");
}

#[test]
fn test_indent_tab_then_space() {
    let mut src = SourceFile::from_string("\t code\n", PathBuf::from("test.c"));
    fix_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "   code\n");
}

#[test]
fn test_spacing_binary_operators() {
    let mut src = SourceFile::from_string("int x=1+2*3;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x = 1 + 2 * 3"));
}

#[test]
fn test_spacing_comma() {
    let mut src = SourceFile::from_string("void f(int a,int b,int c){}\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int a, int b, int c"));
}

#[test]
fn test_spacing_compound_assign() {
    let mut src = SourceFile::from_string("x+=1;y-=2;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x += 1") && src.content.contains("y -= 2"));
}

#[test]
fn test_spacing_comparison() {
    let mut src = SourceFile::from_string("if (x==1&&y!=2){}\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x == 1") && src.content.contains("y != 2"));
}

#[test]
fn test_spacing_preserve_preprocessor() {
    let mut src = SourceFile::from_string("#define A 1+2\nint x=3;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("#define A 1+2"));
    assert!(src.content.contains("x = 3"));
}

#[test]
fn test_spacing_preserve_comment_line() {
    let mut src = SourceFile::from_string("// x=1+2\nint y=3;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("// x=1+2"));
    assert!(src.content.contains("y = 3"));
}

#[test]
fn test_spacing_disabled() {
    let mut config = FormatConfig::default();
    config.spaces_around_operators = false;
    let mut src = SourceFile::from_string("int x=1+2;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &config).unwrap();
    assert!(src.content.contains("x=1+2"));
}

#[test]
fn test_spacing_empty() {
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_spacing_no_trailing_newline() {
    let mut src = SourceFile::from_string("int x=1;", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x = 1"));
}

#[test]
fn test_spacing_string_literal_preserved() {
    let mut src = SourceFile::from_string("char* s = \"a,b=1\";\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("\"a,b=1\""));
}

#[test]
fn test_spacing_bitwise_ops() {
    let mut src = SourceFile::from_string("x=a&b|c^d;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("a & b") && src.content.contains("b | c") && src.content.contains("c ^ d"));
}

#[test]
fn test_spacing_shift_ops() {
    let mut src = SourceFile::from_string("x=1<<2>>3;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("1 << 2") && src.content.contains("2 >> 3"));
}

#[test]
fn test_spacing_semicolon_after_paren() {
    let mut src = SourceFile::from_string("if (a){}\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "if (a){}\n");
}

#[test]
fn test_no_space_in_for() {
    let mut src = SourceFile::from_string("for (i=0;i<10;i++) {}\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("i = 0") || src.content.contains("i=0"));
}

#[test]
fn test_spacing_increment_decrement() {
    let mut src = SourceFile::from_string("i++;++i;i--;--i;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("i++"));
    assert!(src.content.contains("++i"));
    assert!(src.content.contains("i--"));
    assert!(src.content.contains("--i"));
}

#[test]
fn test_spacing_unary_minus() {
    let mut src = SourceFile::from_string("x=-1;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x = -1"));
}

#[test]
fn test_spacing_arrow_operator() {
    let mut src = SourceFile::from_string("ptr->member;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("ptr->member"));
}

#[test]
fn test_spacing_shift_assign() {
    let mut src = SourceFile::from_string("x<<=1;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x <<= 1"));
}

#[test]
fn test_spacing_address_of() {
    let mut src = SourceFile::from_string("p=&x;\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("p = &x"));
}

#[test]
fn test_spacing_for_semicolons() {
    let mut src = SourceFile::from_string("for(i=0;i<10;i++){}\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("; i"));
}

#[test]
fn test_spacing_space_before_paren_enabled() {
    let mut config = FormatConfig::default();
    config.space_before_paren = true;
    let mut src = SourceFile::from_string("if(x){}\n", PathBuf::from("test.c"));
    fix_spacing(&mut src, &config).unwrap();
    assert!(src.content.contains("if (x)"));
}

#[test]
fn test_brace_attach_function() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("void f()\n{\n  return;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "void f() {\n  return;\n}\n");
}

#[test]
fn test_brace_attach_if() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("if (x)\n{\n  y();\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "if (x) {\n  y();\n}\n");
}

#[test]
fn test_brace_attach_else() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("if (x) {\n} else\n{\n  y();\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.contains("} else {"));
}

#[test]
fn test_brace_attach_for() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("for (;;)\n{\n  break;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "for (;;) {\n  break;\n}\n");
}

#[test]
fn test_brace_attach_while() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("while (1)\n{\n  break;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "while (1) {\n  break;\n}\n");
}

#[test]
fn test_brace_attach_struct() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("struct S\n{\n  int x;\n};\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "struct S {\n  int x;\n};\n");
}

#[test]
fn test_brace_attach_enum() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("enum E\n{\n  A,\n  B\n};\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "enum E {\n  A,\n  B\n};\n");
}

#[test]
fn test_brace_already_attached() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("void f() {\n  return;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "void f() {\n  return;\n}\n");
}

#[test]
fn test_brace_breakout_style_no_change() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Breakout;
    let mut src = SourceFile::from_string("void f() {\n  return;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert_eq!(src.content, "void f()\n{\n  return;\n}\n");
}

#[test]
fn test_brace_attach_preserves_indented_brace() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("void f()\n  {\n  return;\n  }\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.starts_with("void f() {"));
}

#[test]
fn test_brace_attach_switch() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("switch (x)\n{\n  case 1:\n    break;\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.starts_with("switch (x) {"));
}

#[test]
fn test_brace_attach_do_while() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("do\n{\n  x--;\n} while (x > 0);\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.starts_with("do {"));
    assert!(src.content.contains("} while (x > 0);"));
}

#[test]
fn test_brace_attach_breakout_hybrid() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::AttachBreakout;
    let mut src = SourceFile::from_string(
        "void f()\n{\n  if (x)\n  {\n    return;\n  }\n}\nstruct S\n{\n  int x;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.contains("void f() {"), "functions should attach");
    assert!(src.content.contains("if (x) {"), "control flow should attach");
    assert!(src.content.contains("struct S\n{"), "structs should breakout");
}

#[test]
fn test_brace_attach_preserves_block_comment() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("/* comment\n{\n*/\nvoid f()\n{\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.contains("/* comment\n{\n*/"), "block comment with brace should be preserved");
    assert!(src.content.contains("void f() {"), "real brace should still attach");
}

#[test]
fn test_brace_attach_preserves_string_with_brace() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Attach;
    let mut src = SourceFile::from_string("char* s = \"{\";\nvoid f()\n{\n}\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.contains("\"{\""), "string literal with brace preserved");
    assert!(src.content.contains("void f() {"), "real brace attached");
}

#[test]
fn test_brace_breakout_struct() {
    let mut config = FormatConfig::default();
    config.brace_style = BraceStyle::Breakout;
    let mut src = SourceFile::from_string("struct S {\n  int x;\n};\n", PathBuf::from("test.c"));
    fix_braces(&mut src, &config).unwrap();
    assert!(src.content.contains("struct S\n{"), "struct brace should breakout");
}

#[test]
fn test_blank_collapse_consecutive() {
    let mut config = FormatConfig::default();
    config.max_consecutive_blank_lines = 2;
    let mut src = SourceFile::from_string("int x;\n\n\n\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\n\nint y;\n");
}

#[test]
fn test_blank_collapse_to_one() {
    let mut config = FormatConfig::default();
    config.max_consecutive_blank_lines = 1;
    let mut src = SourceFile::from_string("int x;\n\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\nint y;\n");
}

#[test]
fn test_blank_after_include() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 1;
    let mut src = SourceFile::from_string("#include <stdio.h>\n#include <stdlib.h>\nint main() {}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("#include <stdlib.h>\n\nint main()"));
}

#[test]
fn test_blank_after_include_two() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 2;
    let mut src = SourceFile::from_string("#include <stdio.h>\nint x;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("#include <stdio.h>\n\n\nint x;"));
}

#[test]
fn test_blank_leading_removed() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("\n\nint x;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.starts_with("int x;"));
}

#[test]
fn test_blank_trailing_removed() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n\n\n\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.ends_with("int x;\n"));
}

#[test]
fn test_blank_before_function() {
    let mut config = FormatConfig::default();
    config.blank_lines_before_function = 1;
    let mut src = SourceFile::from_string("int x;\nvoid f() {\n}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("int x;\n\nvoid f()"));
}

#[test]
fn test_blank_no_change_needed() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\nint y;\n");
}

#[test]
fn test_blank_empty_input() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_blank_only_whitespace_lines() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n   \n   \nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n\nint y;\n");
}

#[test]
fn test_blank_include_block_multiple_groups() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 1;
    let mut src = SourceFile::from_string(
        "#include <stdio.h>\n#include <stdlib.h>\n\n#include \"my.h\"\nint x;\n",
        PathBuf::from("test.c"),
    );
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("#include \"my.h\"\n\nint x;"));
}

#[test]
fn test_blank_preserve_single_newline_ending() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\n");
}

#[test]
fn test_blank_no_trailing_newline_input() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x;", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;");
}

#[test]
fn test_blank_collapse_max_zero() {
    let mut config = FormatConfig::default();
    config.max_consecutive_blank_lines = 0;
    let mut src = SourceFile::from_string("int x;\n\n\nint y;\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\nint y;\n");
}

#[test]
fn test_blank_before_function_zero() {
    let mut config = FormatConfig::default();
    config.blank_lines_before_function = 0;
    let mut src = SourceFile::from_string("int x;\n\nvoid f() {\n}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\nvoid f() {\n}\n");
}

#[test]
fn test_blank_include_at_file_end() {
    let mut config = FormatConfig::default();
    config.blank_lines_after_include = 1;
    let mut src = SourceFile::from_string("#include <stdio.h>\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "#include <stdio.h>\n");
}

#[test]
fn test_blank_function_def_not_matching_for_loop() {
    let mut config = FormatConfig::default();
    config.blank_lines_before_function = 1;
    let mut src = SourceFile::from_string("int x;\nfor (i = 0; i < 10; i++) {\n}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert_eq!(src.content, "int x;\nfor (i = 0; i < 10; i++) {\n}\n");
}

#[test]
fn test_blank_consecutive_functions() {
    let mut config = FormatConfig::default();
    config.blank_lines_before_function = 1;
    let mut src = SourceFile::from_string("void f() {\n}\nvoid g() {\n}\n", PathBuf::from("test.c"));
    fix_blank_lines(&mut src, &config).unwrap();
    assert!(src.content.contains("}\n\nvoid g()"));
}

#[test]
fn test_comment_single_line_block() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x; /* comment */ int y;\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("// comment"));
    assert!(!src.content.contains("/*"));
}

#[test]
fn test_comment_standalone_block() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/* standalone comment */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.starts_with("// standalone comment"));
}

#[test]
fn test_comment_multi_line_block() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/* line1\n   line2\n   line3 */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[0].starts_with("//"));
    assert!(lines[1].trim_start().starts_with("//"));
    assert!(lines[2].trim_start().starts_with("//"));
}

#[test]
fn test_comment_preserve_double_slash() {
    let config = FormatConfig::default();
    let input = "// already slash comment\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_comment_copyright_block() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/* Copyright 2026 My Corp\n * All rights reserved. */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("// Copyright 2026 My Corp"));
    assert!(src.content.contains("// All rights reserved."));
}

#[test]
fn test_comment_preserve_mode() {
    let mut config = FormatConfig::default();
    config.comment_style = CommentStyle::Preserve;
    let input = "/* keep this block comment */\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_comment_string_literal_not_converted() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("char* s = \"/* not a comment */\";\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("\"/* not a comment */\""));
}

#[test]
fn test_comment_empty_block() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/**/\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert_eq!(src.content, "//\n");
}

#[test]
fn test_comment_adjacent_blocks() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/* a */ /* b */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("// a"));
    assert!(src.content.contains("// b"));
    assert!(!src.content.contains("/*"));
}

#[test]
fn test_comment_empty_input() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_comment_multi_line_with_stars() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/**\n * Copyright\n * License\n */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("// Copyright"));
    assert!(src.content.contains("// License"));
    assert!(!src.content.contains("/**"));
}

#[test]
fn test_comment_inline_preserves_code() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("int x = 1; /* TODO: fix */ int y = 2;\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("int x = 1;"));
    assert!(src.content.contains("// TODO: fix"));
    assert!(src.content.contains("int y = 2;"));
}

#[test]
fn test_comment_char_literal_not_converted() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("char c = '/';\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert_eq!(src.content, "char c = '/';\n");
}

#[test]
fn test_comment_preserves_inner_indentation() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/* Args:\n *   x - the x\n *   y - the y\n */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("// Args:"));
    assert!(src.content.contains("//   x - the x"));
    assert!(src.content.contains("//   y - the y"));
}

#[test]
fn test_comment_star_emphasis_preserved() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("/* ***important*** */\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("// ***important***"));
}

#[test]
fn test_comment_escaped_quote_in_string() {
    let config = FormatConfig::default();
    let mut src = SourceFile::from_string("char* s = \"a\\\"/* not comment */\\\"b\";\n", PathBuf::from("test.c"));
    fix_comments(&mut src, &config).unwrap();
    assert!(src.content.contains("/* not comment */"));
}

#[test]
fn test_pointer_left_align() {
    let mut src = SourceFile::from_string("int *p;\nchar *s;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int* p"), "got: {}", src.content);
    assert!(src.content.contains("char* s"), "got: {}", src.content);
}

#[test]
fn test_pointer_right_align() {
    let mut config = FormatConfig::default();
    config.pointer_alignment = PointerAlignment::Right;
    let mut src = SourceFile::from_string("int* p;\nchar* s;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &config).unwrap();
    assert!(src.content.contains("int *p"), "got: {}", src.content);
    assert!(src.content.contains("char *s"), "got: {}", src.content);
}

#[test]
fn test_pointer_no_change_when_correct() {
    let mut src = SourceFile::from_string("int* p;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "int* p;\n");
}

#[test]
fn test_double_pointer_left() {
    let mut src = SourceFile::from_string("int **pp;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int** pp"), "got: {}", src.content);
}

#[test]
fn test_double_pointer_right() {
    let mut config = FormatConfig::default();
    config.pointer_alignment = PointerAlignment::Right;
    let mut src = SourceFile::from_string("int** pp;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &config).unwrap();
    assert!(src.content.contains("int **pp"), "got: {}", src.content);
}

#[test]
fn test_pointer_skip_preprocessor() {
    let mut src = SourceFile::from_string("#define PTR int*\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "#define PTR int*\n");
}

#[test]
fn test_pointer_skip_comment() {
    let mut src = SourceFile::from_string("// int *p;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "// int *p;\n");
}

#[test]
fn test_pointer_in_string_literal() {
    let mut src = SourceFile::from_string("char* s = \"int *p\";\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("\"int *p\""), "got: {}", src.content);
}

#[test]
fn test_pointer_empty_input() {
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_pointer_no_trailing_newline() {
    let mut src = SourceFile::from_string("int *p;", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int* p"));
}

#[test]
fn test_pointer_double_pointer_left() {
    let mut src = SourceFile::from_string("int **pp;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int** pp"), "got: {}", src.content);
}

#[test]
fn test_pointer_double_pointer_right() {
    let mut config = FormatConfig::default();
    config.pointer_alignment = PointerAlignment::Right;
    let mut src = SourceFile::from_string("int** pp;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &config).unwrap();
    assert!(src.content.contains("int **pp"), "got: {}", src.content);
}

#[test]
fn test_pointer_no_corrupt_multiplication() {
    let mut src = SourceFile::from_string("int result = x * y;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("x * y"), "got: {}", src.content);
}

#[test]
fn test_pointer_no_corrupt_dereference() {
    let mut src = SourceFile::from_string("return *ptr;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("return *ptr"), "got: {}", src.content);
}

#[test]
fn test_pointer_no_corrupt_if_deref() {
    let mut src = SourceFile::from_string("if (*ptr == 1) {}\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("if (*ptr"), "got: {}", src.content);
}

#[test]
fn test_pointer_block_comment_protected() {
    let mut src = SourceFile::from_string("/* int *p; */\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int *p"), "got: {}", src.content);
}

#[test]
fn test_pointer_multiline_block_comment_protected() {
    let mut src = SourceFile::from_string("/*\n * int *p;\n */\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int *p"), "got: {}", src.content);
}

#[test]
fn test_pointer_no_space_normalized_left() {
    let mut src = SourceFile::from_string("int*p;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int* p"), "got: {}", src.content);
}

#[test]
fn test_pointer_no_space_normalized_right() {
    let mut config = FormatConfig::default();
    config.pointer_alignment = PointerAlignment::Right;
    let mut src = SourceFile::from_string("int*p;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &config).unwrap();
    assert!(src.content.contains("int *p"), "got: {}", src.content);
}

#[test]
fn test_pointer_const_qualified() {
    let mut src = SourceFile::from_string("const char *s;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("char* s"), "got: {}", src.content);
}

#[test]
fn test_pointer_function_pointer_unchanged() {
    let mut src = SourceFile::from_string("int (*func)(int);\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int (*func)(int)"), "got: {}", src.content);
}

#[test]
fn test_pointer_cast_unchanged() {
    let mut src = SourceFile::from_string("void* p = (int*)ptr;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("(int*)ptr"), "cast should be unchanged, got: {}", src.content);
}

#[test]
fn test_pointer_multi_decl() {
    let mut src = SourceFile::from_string("int *a, *b;\n", PathBuf::from("test.c"));
    fix_pointer_style(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int* a"), "got: {}", src.content);
    assert!(src.content.contains("int* b") || src.content.contains("*b"), "got: {}", src.content);
}

use cclinter::formatter::switch_indent::fix_switch_indent;

#[test]
fn test_switch_case_indent_enabled() {
    let mut src = SourceFile::from_string(
        "switch (x) {\ncase 1:\nbreak;\ncase 2:\nbreak;\ndefault:\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1:"), "got: {}", src.content);
    assert!(src.content.contains("    break;"), "got: {}", src.content);
}

use cclinter::formatter::alignment::fix_alignment;

#[test]
fn test_struct_field_alignment() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  int x;\n  char* name;\n  float value;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[1].contains("int   x;") || lines[1].contains("int  x;"), "got: {}", lines[1]);
    assert!(src.content.contains("char* name;"), "got: {}", src.content);
    assert!(src.content.contains("float value;"), "got: {}", src.content);
}

#[test]
fn test_enum_value_alignment() {
    let mut src = SourceFile::from_string(
        "enum Bar {\n  FOO = 1,\n  BAZ = 2,\n  LONG_NAME = 3,\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("FOO       = 1"), "got: {}", src.content);
    assert!(src.content.contains("BAZ       = 2"), "got: {}", src.content);
    assert!(src.content.contains("LONG_NAME = 3"), "got: {}", src.content);
}

#[test]
fn test_struct_no_alignment_needed() {
    let input = "struct S {\n  int   x;\n  char* y;\n};\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_struct_with_comments() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  // comment\n  int x;\n  char* name;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("// comment"));
}

#[test]
fn test_enum_without_values() {
    let input = "enum E {\n  A,\n  B,\n  C,\n};\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_alignment_empty_input() {
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_struct_multiline_type() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  unsigned int count;\n  char* name;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("unsigned int count;"), "got: {}", src.content);
    assert!(src.content.contains("char*        name;") || src.content.contains("char*       name;") || src.content.contains("char*  name;"), "got: {}", src.content);
}

#[test]
fn test_nested_struct() {
    let mut src = SourceFile::from_string(
        "struct Outer {\n  int x;\n  struct Inner {\n    int a;\n    char* b;\n  } inner;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int  x;") || src.content.contains("int x;"), "outer got: {}", src.content);
}

#[test]
fn test_alignment_no_trailing_newline() {
    let mut src = SourceFile::from_string("struct S {\n  int x;\n  char* y;\n};", PathBuf::from("test.c"));
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(!src.content.ends_with('\n'));
}

#[test]
fn test_struct_preserves_field_with_initializer() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  int x;\n  int* ptr;\n  int count = 0;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("count = 0"), "got: {}", src.content);
    assert!(src.content.contains("int  x;") || src.content.contains("int x;"), "x not corrupted, got: {}", src.content);
    assert!(src.content.contains("int* ptr;") || src.content.contains("int*  ptr;"), "ptr not corrupted, got: {}", src.content);
}

#[test]
fn test_struct_array_field_aligned() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  int x;\n  char name[32];\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("name[32]"), "array field preserved, got: {}", src.content);
    let lines: Vec<&str> = src.content.lines().collect();
    let x_col = lines[1].find('x').unwrap_or(0);
    let n_col = lines[2].find("name").unwrap_or(0);
    assert_eq!(x_col, n_col, "field names should start at same column, got x@{} name@{}", x_col, n_col);
}

#[test]
fn test_struct_bit_field_skipped() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  int x : 3;\n  int y : 5;\n  int z;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains(": 3"), "bit field preserved, got: {}", src.content);
    assert!(src.content.contains(": 5"), "bit field preserved, got: {}", src.content);
}

#[test]
fn test_struct_initializer_not_corrupted() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  int a;\n  int b;\n  int c = 42;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("a;"), "got: {}", src.content);
    assert!(src.content.contains("b;"), "got: {}", src.content);
    assert!(src.content.contains("c = 42"), "got: {}", src.content);
    assert!(!src.content.contains("a ="), "a should not be corrupted, got: {}", src.content);
    assert!(!src.content.contains("b ="), "b should not be corrupted, got: {}", src.content);
}

#[test]
fn test_struct_block_comment_brace() {
    let mut src = SourceFile::from_string(
        "struct Foo { /* } */\n  int x;\n  char* y;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("int"), "got: {}", src.content);
    assert!(src.content.contains("char*"), "got: {}", src.content);
}

#[test]
fn test_struct_comment_in_string() {
    let mut src = SourceFile::from_string(
        "struct Foo {\n  int x; // real comment\n  char* name;\n};\n",
        PathBuf::from("test.c"),
    );
    fix_alignment(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("// real comment"), "got: {}", src.content);
}

#[test]
fn test_switch_case_indent_disabled() {
    let mut config = FormatConfig::default();
    config.switch_case_indent = false;
    let input = "switch (x) {\ncase 1:\nbreak;\n}\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_switch_indent(&mut src, &config).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_switch_already_indented() {
    let mut src = SourceFile::from_string(
        "switch (x) {\n  case 1:\n  break;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1:"), "got: {}", src.content);
    assert!(src.content.contains("    break;"), "got: {}", src.content);
}

#[test]
fn test_switch_nested() {
    let mut src = SourceFile::from_string(
        "switch (x) {\ncase 1:\nswitch (y) {\ncase 1:\nbreak;\n}\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1:"), "got: {}", src.content);
    assert!(src.content.contains("  switch (y)"), "got: {}", src.content);
    assert!(src.content.contains("    case 1:"), "got: {}", src.content);
    assert!(
        src.content.contains("      break;"),
        "inner break at 6, got: {}",
        src.content
    );
}

#[test]
fn test_switch_with_braces_in_case() {
    let mut src = SourceFile::from_string(
        "switch (x) {\ncase 1: {\nint y = 1;\nbreak;\n}\ndefault:\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1: {"), "got: {}", src.content);
    assert!(src.content.contains("  }"), "inner brace at body_indent, got: {}", src.content);
    assert!(src.content.contains("  default:"), "got: {}", src.content);
}

#[test]
fn test_switch_empty_input() {
    let mut src = SourceFile::from_string("", PathBuf::from("test.c"));
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, "");
}

#[test]
fn test_switch_no_trailing_newline() {
    let mut src =
        SourceFile::from_string("switch (x) {\ncase 1:\nbreak;\n}", PathBuf::from("test.c"));
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1:"));
}

#[test]
fn test_switch_custom_indent_width() {
    let mut config = FormatConfig::default();
    config.indent_width = 4;
    let mut src = SourceFile::from_string(
        "switch (x) {\ncase 1:\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &config).unwrap();
    assert!(src.content.contains("  case 1:"), "got: {}", src.content);
    assert!(src.content.contains("    break;"), "got: {}", src.content);
}

#[test]
fn test_sort_system_headers() {
    let input = "#include <stdio.h>\n#include <stdlib.h>\n#include <assert.h>\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[0].contains("<assert.h>"), "got: {}", lines[0]);
    assert!(lines[1].contains("<stdio.h>"), "got: {}", lines[1]);
    assert!(lines[2].contains("<stdlib.h>"), "got: {}", lines[2]);
}

#[test]
fn test_sort_project_headers() {
    let input = "#include \"foo.h\"\n#include \"bar.h\"\n#include \"baz.h\"\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[0].contains("\"bar.h\""), "got: {}", lines[0]);
    assert!(lines[1].contains("\"baz.h\""), "got: {}", lines[1]);
    assert!(lines[2].contains("\"foo.h\""), "got: {}", lines[2]);
}

#[test]
fn test_three_group_sort() {
    let input = "#include \"foo.h\"\n#include <stdio.h>\n#include \"bar.h\"\n#include <stdlib.h>\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    let mut saw_system = false;
    let mut saw_project = false;
    for line in &lines {
        if line.starts_with("#include <") {
            saw_system = true;
            assert!(!saw_project, "system headers must come before project headers");
        }
        if line.starts_with("#include \"") {
            saw_project = true;
        }
    }
    assert!(saw_system, "should have system headers");
    assert!(saw_project, "should have project headers");
}

#[test]
fn test_corresponding_header_first() {
    let input = "#include \"bar.h\"\n#include \"foo.h\"\n#include <stdio.h>\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("foo.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[0].contains("\"foo.h\""), "corresponding header should be first, got: {}", lines[0]);
}

#[test]
fn test_include_sort_disabled() {
    let input = "#include <stdlib.h>\n#include <stdio.h>\n\nint x;\n";
    let mut config = FormatConfig::default();
    config.include_sorting = IncludeSorting::Disabled;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &config).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[0].contains("<stdlib.h>"), "should not sort when disabled, got: {}", lines[0]);
    assert!(lines[1].contains("<stdio.h>"), "should not sort when disabled, got: {}", lines[1]);
}

#[test]
fn test_include_sort_no_includes() {
    let input = "int x = 1;\nint y = 2;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_include_sort_blank_line_between_groups() {
    let input = "#include <stdio.h>\n#include \"bar.h\"\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    let sys_idx = lines.iter().position(|l| l.starts_with("#include <")).unwrap();
    let proj_idx = lines.iter().position(|l| l.starts_with("#include \"")).unwrap();
    assert!(proj_idx > sys_idx + 1, "should have blank line between groups");
}

#[test]
fn test_sort_skips_conditional_includes() {
    let input = "#include <stdlib.h>\n#ifdef FOO\n#include <stdio.h>\n#endif\n#include <assert.h>\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(
        src.content, input,
        "should not sort when conditional directives between includes"
    );
}

#[test]
fn test_sort_skips_comment_between_includes() {
    let input = "#include <stdlib.h>\n// system headers\n#include <stdio.h>\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    assert_eq!(
        src.content, input,
        "should not sort when comments between includes"
    );
}

#[test]
fn test_sort_path_qualified_project_header() {
    let input = "#include <stdio.h>\n#include \"sub/dir/foo.h\"\n#include \"sub/dir/bar.h\"\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    let sys_idx = lines.iter().position(|l| l.starts_with("#include <")).unwrap();
    let bar_idx = lines.iter().position(|l| l.contains("\"sub/dir/bar.h\"")).unwrap();
    let foo_idx = lines.iter().position(|l| l.contains("\"sub/dir/foo.h\"")).unwrap();
    assert!(bar_idx > sys_idx, "project headers after system");
    assert!(bar_idx < foo_idx, "bar.h before foo.h alphabetically");
}

#[test]
fn test_sort_only_includes_no_code() {
    let input = "#include <stdlib.h>\n#include <stdio.h>\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines[0].contains("<stdio.h>"), "got: {}", lines[0]);
    assert!(lines[1].contains("<stdlib.h>"), "got: {}", lines[1]);
}

#[test]
fn test_sort_preserves_duplicates() {
    let input = "#include <stdio.h>\n#include <stdio.h>\n\nint x;\n";
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_include_sort(&mut src, &FormatConfig::default()).unwrap();
    let count = src.content.matches("#include <stdio.h>").count();
    assert_eq!(count, 2, "should preserve duplicate includes");
}

#[test]
fn test_switch_deref_not_treated_as_comment() {
    let mut src = SourceFile::from_string(
        "switch (x) {\ncase 1:\n*ptr = 1;\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(
        src.content.contains("    *ptr = 1;"),
        "deref should be at content_indent, got: {}",
        src.content
    );
}

#[test]
fn test_switch_empty_one_line() {
    let mut src = SourceFile::from_string(
        "switch (x) {}\nint y = 1;\nswitch (z) {\ncase 1:\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1:"), "got: {}", src.content);
    assert!(src.content.contains("    break;"), "got: {}", src.content);
}

use cclinter::formatter::line_length::fix_line_length;
use cclinter::formatter::include_sort::fix_include_sort;
use cclinter::config::IncludeSorting;

#[test]
fn test_wrap_long_line() {
    let input = "int very_long_variable_name = some_function_with_many_args(arg1, arg2, arg3, arg4, arg5, arg6, arg7);\n";
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    for line in src.content.lines() {
        assert!(
            line.chars().count() <= 80,
            "Line too long: {} (len={})",
            line,
            line.chars().count()
        );
    }
}

#[test]
fn test_no_break_inside_string_literal() {
    let long_str = "x".repeat(100);
    let input = format!("char *s = \"{}\";\n", long_str);
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert!(
        src.content.contains(&long_str),
        "String literal should not be split, got: {}",
        src.content
    );
}

#[test]
fn test_no_break_preprocessor_directive() {
    let input = "#include \"some/very/deeply/nested/header/path/that/exceeds/the/column/limit.h\"\n";
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert_eq!(
        src.content, input,
        "Preprocessor directive should not be wrapped"
    );
}

#[test]
fn test_no_wrap_line_at_exact_limit() {
    let content = "x".repeat(80);
    let input = format!("{}\n", content);
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert_eq!(
        src.content, input,
        "Line at exact limit should not be wrapped"
    );
}

#[test]
fn test_no_hard_split_long_identifier() {
    let ident = "x".repeat(100);
    let input = format!("int {} = 0;\n", ident);
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert!(
        src.content.contains(&ident),
        "Long identifier should not be hard-split, got: {}",
        src.content
    );
}

#[test]
fn test_line_length_empty_input() {
    let input = "";
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert_eq!(src.content, "", "Empty input should remain empty");
}

#[test]
fn test_line_length_whitespace_only_line() {
    let input = "    \n";
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert_eq!(src.content, input, "Whitespace-only line should be unchanged");
}

#[test]
fn test_no_wrap_short_line() {
    let input = "int x = 1;\n";
    let mut config = FormatConfig::default();
    config.column_limit = 80;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    assert_eq!(src.content, input);
}

#[test]
fn test_wrap_preserves_indent() {
    let input = "    int result = very_long_function_name_that_exceeds_the_column_limit_by_a_lot(a, b);\n";
    let mut config = FormatConfig::default();
    config.column_limit = 60;
    let mut src = SourceFile::from_string(input, PathBuf::from("test.c"));
    fix_line_length(&mut src, &config).unwrap();
    let lines: Vec<&str> = src.content.lines().collect();
    assert!(lines.len() > 1, "Should have wrapped into multiple lines");
    for line in &lines[1..] {
        assert!(
            line.starts_with("    "),
            "Continuation should preserve base indent, got: {}",
            line
        );
    }
}

#[test]
fn test_switch_block_comment_brace_not_counted() {
    let mut src = SourceFile::from_string(
        "switch (x) { /* } */\ncase 1:\nbreak;\n}\n",
        PathBuf::from("test.c"),
    );
    fix_switch_indent(&mut src, &FormatConfig::default()).unwrap();
    assert!(src.content.contains("  case 1:"), "got: {}", src.content);
    assert!(src.content.contains("    break;"), "got: {}", src.content);
}
