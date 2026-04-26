use cclinter::checker::naming::check_naming;
use cclinter::common::source::SourceFile;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

#[test]
fn test_snake_case_function() {
    disable_color();
    let input = "void BadFunction() {}\nvoid good_function() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "snake_case", "function");
    assert!(diags.iter().any(|d| d.message.contains("BadFunction")));
}

#[test]
fn test_upper_snake_macro() {
    disable_color();
    let input = "#define bad_macro 1\n#define GOOD_MACRO 2\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "upper_snake_case", "macro");
    assert!(diags.iter().any(|d| d.message.contains("bad_macro")));
}

#[test]
fn test_pascal_type() {
    disable_color();
    let input = "typedef struct bad_type {} bad_type;\ntypedef struct GoodType {} GoodType;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "pascal_case", "type");
    assert!(diags.iter().any(|d| d.message.contains("bad_type")));
}

#[test]
fn test_snake_case_function_good() {
    disable_color();
    let input = "void good_function() {}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "snake_case", "function");
    assert!(diags.is_empty());
}

#[test]
fn test_upper_snake_macro_good() {
    disable_color();
    let input = "#define GOOD_MACRO 1\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "upper_snake_case", "macro");
    assert!(diags.is_empty());
}

#[test]
fn test_unknown_kind_returns_empty() {
    disable_color();
    let src = SourceFile::from_string("anything", PathBuf::from("test.c"));
    let diags = check_naming(&src, "snake_case", "unknown");
    assert!(diags.is_empty());
}

#[test]
fn test_constant_check() {
    disable_color();
    let input = "const int badConst = 1;\nconst int GOOD_CONST = 2;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_naming(&src, "upper_snake_case", "constant");
    assert!(diags.iter().any(|d| d.message.contains("badConst")));
}

#[test]
fn test_empty_input() {
    disable_color();
    let src = SourceFile::from_string("", PathBuf::from("test.c"));
    let diags = check_naming(&src, "snake_case", "function");
    assert!(diags.is_empty());
}
