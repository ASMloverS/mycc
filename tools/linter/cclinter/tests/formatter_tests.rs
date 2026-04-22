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
