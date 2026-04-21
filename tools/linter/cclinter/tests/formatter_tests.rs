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
use cclinter::config::FormatConfig;
use cclinter::formatter::encoding::fix_encoding;
use cclinter::formatter::indent::fix_indent;
use cclinter::formatter::spacing::fix_spacing;
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
