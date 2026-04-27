use cclinter::checker::unused::check_unused;
use cclinter::common::source::SourceFile;
use cclinter::config::UnusedConfig;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

fn default_config() -> UnusedConfig {
    UnusedConfig::default()
}

#[test]
fn test_unused_variable() {
    disable_color();
    let input = "void f() {\n  int x = 1;\n  int y = x + 1;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("y")));
}

#[test]
fn test_unused_macro() {
    disable_color();
    let input = "#define UNUSED_MACRO 42\nint main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-macro"));
}

#[test]
fn test_used_variable_not_flagged() {
    disable_color();
    let input = "int f() {\n  int x = 1;\n  return x;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_used_macro_not_flagged() {
    disable_color();
    let input = "#define USED_MACRO 42\nint x = USED_MACRO;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-macro"));
}

#[test]
fn test_empty_file() {
    disable_color();
    let src = SourceFile::from_string("", PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_multiple_unused_variables() {
    disable_color();
    let input = "void f() {\n  int a = 1;\n  int b = 2;\n  int c = 3;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    let unused_vars: Vec<&str> = diags
        .iter()
        .filter(|d| d.rule_id == "bugprone-unused-variable")
        .map(|d| d.message.as_str())
        .collect();
    assert!(unused_vars.iter().any(|m| m.contains("a")));
    assert!(unused_vars.iter().any(|m| m.contains("b")));
    assert!(unused_vars.iter().any(|m| m.contains("c")));
}

#[test]
fn test_unused_variable_no_initializer() {
    disable_color();
    let input = "void f() {\n  int x;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_function_decl_not_flagged_as_unused_var() {
    disable_color();
    let input = "void foo();\nint bar(int x);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("foo")));
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("bar")));
}

#[test]
fn test_static_function_not_flagged() {
    disable_color();
    let input = "static int helper() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("helper")));
}

#[test]
fn test_compound_type_detected() {
    disable_color();
    let input = "void f() {\n  unsigned int x = 1;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_const_qualifier_detected() {
    disable_color();
    let input = "void f() {\n  const int x = 1;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_variable_in_comment_not_counted() {
    disable_color();
    let input = "void f() {\n  int x = 1; /* x is used here */\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_variable_in_string_not_counted() {
    disable_color();
    let input = "void f() {\n  int x = 1;\n  char* s = \"x\";\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_config_disabled() {
    disable_color();
    let config = UnusedConfig { enabled: false };
    let input = "void f() {\n  int x = 1;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &config);
    assert!(diags.is_empty());
}

#[test]
fn test_long_long_type() {
    disable_color();
    let input = "void f() {\n  long long x = 1;\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_pointer_declaration() {
    disable_color();
    let input = "int f() {\n  int x = 1;\n  int* p = &x;\n  return x;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("p")));
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-variable" && d.message.contains("x")));
}

#[test]
fn test_macro_in_ifdef_not_flagged() {
    disable_color();
    let input = "#define MY_MACRO 42\n#ifdef MY_MACRO\nint x = 1;\n#endif\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_unused(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-unused-macro" && d.message.contains("MY_MACRO")));
}
