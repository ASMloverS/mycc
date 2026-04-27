use cclinter::checker::complexity::check_complexity;
use cclinter::common::source::SourceFile;
use cclinter::config::ComplexityConfig;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

fn default_config() -> ComplexityConfig {
    ComplexityConfig {
        max_function_lines: 100,
        max_file_lines: 2000,
        max_nesting_depth: 5,
    }
}

#[test]
fn test_function_too_long() {
    disable_color();
    let lines: Vec<String> = (0..110).map(|i| format!("  int x{} = {};", i, i)).collect();
    let input = format!("void long_fn() {{\n{}\n}}\n", lines.join("\n"));
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_file_too_long() {
    disable_color();
    let lines: Vec<String> = (0..2100).map(|i| format!("int x{} = {};", i, i)).collect();
    let input = lines.join("\n") + "\n";
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "readability-file-size"));
}

#[test]
fn test_deep_nesting() {
    disable_color();
    let input = "void f() {\n  if (1) {\n    if (2) {\n      if (3) {\n        if (4) {\n          if (5) {\n            if (6) {\n            }\n          }\n        }\n      }\n    }\n  }\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
}

#[test]
fn test_function_within_limit() {
    disable_color();
    let input = "void short_fn() {\n  int x = 1;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_file_within_limit() {
    disable_color();
    let input = "int x = 1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-file-size"));
}

#[test]
fn test_nesting_within_limit() {
    disable_color();
    let input = "void f() {\n  if (1) {\n    if (2) {\n    }\n  }\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
}

#[test]
fn test_empty_file() {
    disable_color();
    let src = SourceFile::from_string("", PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(diags.is_empty());
}

#[test]
fn test_custom_config_thresholds() {
    disable_color();
    let config = ComplexityConfig {
        max_function_lines: 5,
        max_file_lines: 500,
        max_nesting_depth: 2,
    };
    let input = "void f() {\n  if (1) {\n    if (2) {\n    }\n  }\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
}

#[test]
fn test_static_function_detected() {
    disable_color();
    let lines: Vec<String> = (0..110).map(|i| format!("  int x{} = {};", i, i)).collect();
    let input = format!("static void long_fn() {{\n{}\n}}\n", lines.join("\n"));
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_allman_brace_style() {
    disable_color();
    let lines: Vec<String> = (0..110).map(|i| format!("  int x{} = {};", i, i)).collect();
    let input = format!("void long_fn()\n{{\n{}\n}}\n", lines.join("\n"));
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_allman_short_function_ok() {
    disable_color();
    let input = "void f()\n{\n  int x = 1;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_single_line_function_at_eof() {
    disable_color();
    let input = "void f() { return 0; }";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_multiple_functions() {
    disable_color();
    let config = ComplexityConfig {
        max_function_lines: 5,
        max_file_lines: 2000,
        max_nesting_depth: 5,
    };
    let mut input = String::new();
    for fn_idx in 0..3 {
        input.push_str(&format!("void fn_{}() {{\n", fn_idx));
        for j in 0..10 {
            input.push_str(&format!("  int x{} = {};\n", j, j));
        }
        input.push_str("}\n");
    }
    let src = SourceFile::from_string(&input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &config);
    let fn_diags: Vec<_> = diags.iter().filter(|d| d.rule_id == "readability-function-size").collect();
    assert_eq!(fn_diags.len(), 3);
}

#[test]
fn test_exact_boundary_function() {
    disable_color();
    let config = ComplexityConfig {
        max_function_lines: 5,
        max_file_lines: 2000,
        max_nesting_depth: 5,
    };
    let input = "void f() {\n  int a = 1;\n  int b = 2;\n  int c = 3;\n  int d = 4;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &config);
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_one_over_boundary_function() {
    disable_color();
    let config = ComplexityConfig {
        max_function_lines: 5,
        max_file_lines: 2000,
        max_nesting_depth: 5,
    };
    let input = "void f() {\n  int a = 1;\n  int b = 2;\n  int c = 3;\n  int d = 4;\n  int e = 5;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_nesting_transition_emits_once() {
    disable_color();
    let config = ComplexityConfig {
        max_function_lines: 100,
        max_file_lines: 2000,
        max_nesting_depth: 2,
    };
    let input = "void f() {\n  if (1) {\n    if (2) {\n      int a = 1;\n      int b = 2;\n    }\n  }\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &config);
    let nesting_diags: Vec<_> = diags.iter().filter(|d| d.rule_id == "readability-deep-nesting").collect();
    assert_eq!(nesting_diags.len(), 1);
}

#[test]
fn test_unsigned_int_function() {
    disable_color();
    let input = "unsigned int compute() {\n  return 0;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_extern_function() {
    disable_color();
    let input = "extern int api_call() {\n  return 0;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_struct_return_type_function() {
    disable_color();
    let input = "struct point create_point() {\n  struct point p;\n  return p;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}

#[test]
fn test_const_qualifier_function() {
    disable_color();
    let input = "const char* get_name() {\n  return \"hello\";\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = check_complexity(&src, &default_config());
    assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
}
