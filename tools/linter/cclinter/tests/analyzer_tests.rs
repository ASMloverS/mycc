use cclinter::analyzer::{analyze_source, basic, strict};
use cclinter::common::source::SourceFile;
use cclinter::config::{AnalysisConfig, AnalysisLevel};
use std::path::PathBuf;

mod common;

use std::process::Command;

fn bin() -> std::path::PathBuf {
    common::get_bin()
}

#[test]
fn test_non_void_missing_return() {
    let input = "int foo() { int x = 1; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_implicit_int_conversion() {
    let input = "float x = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_void_no_return_ok() {
    let input = "void foo() { return; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_non_void_with_return_ok() {
    let input = "int foo() { return 1; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_float_with_decimal_ok() {
    let input = "float x = 3.14;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_double_implicit_conversion() {
    let input = "double y = 100;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_missing_return_multiline() {
    let input = "int bar(int a) {\n  int b = a + 1;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_empty_source_basic() {
    let input = "";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.is_empty());
}

#[test]
fn test_none_level_no_diags() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = AnalysisConfig::default();
    let diags = analyze_source(&src, &AnalysisLevel::None, &config);
    assert!(diags.is_empty());
}

#[test]
fn test_basic_level_runs() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = AnalysisConfig::default();
    let _ = analyze_source(&src, &AnalysisLevel::Basic, &config);
}

#[test]
fn test_strict_includes_basic() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = AnalysisConfig::default();
    let basic_diags = analyze_source(&src, &AnalysisLevel::Basic, &config);
    let strict_diags = analyze_source(&src, &AnalysisLevel::Strict, &config);
    assert!(strict_diags.len() >= basic_diags.len());
}

#[test]
fn test_deep_includes_strict() {
    let input = "int main() { return 0; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = AnalysisConfig::default();
    let strict_diags = analyze_source(&src, &AnalysisLevel::Strict, &config);
    let deep_diags = analyze_source(&src, &AnalysisLevel::Deep, &config);
    assert!(deep_diags.len() >= strict_diags.len());
}

#[test]
fn test_empty_source() {
    let input = "";
    let src = SourceFile::from_string(input, PathBuf::from("empty.c"));
    let config = AnalysisConfig::default();
    let diags = analyze_source(&src, &AnalysisLevel::Deep, &config);
    assert!(diags.is_empty());
}

#[test]
fn test_static_non_void_missing_return() {
    let input = "static int foo() { int x = 1; }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_return_inside_nested_braces() {
    let input = "int foo() { if (cond) { return 1; } }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-return"));
}

#[test]
fn test_implicit_conv_comment_ignored() {
    let input = "// float x = 42;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_implicit_conv_zero_ok() {
    let input = "float x = 0;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_implicit_conv_negative() {
    let input = "float x = -1;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-implicit-conversion"));
}

#[test]
fn test_uninit_var_detected() {
    let input = "int main() {\n  int x;\n  printf(\"%d\", x);\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-uninit"));
}

#[test]
fn test_uninit_var_assigned_ok() {
    let input = "int main() {\n  int x;\n  x = 5;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-uninit"));
}

#[test]
fn test_uninit_var_init_ok() {
    let input = "int main() {\n  int x = 5;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = basic::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-uninit"));
}

#[test]
fn test_analysis_level_none() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.c"), "int main(void) { return 0; }\n").unwrap();
    let output = Command::new(bin())
        .args(["--analysis-level", "none", dir.path().join("a.c").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_analysis_level_basic() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.c"), "int main(void) { return 0; }\n").unwrap();
    let output = Command::new(bin())
        .args(["--analysis-level", "basic", dir.path().join("a.c").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_analysis_level_strict() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.c"), "int main(void) { return 0; }\n").unwrap();
    let output = Command::new(bin())
        .args(["--analysis-level", "strict", dir.path().join("a.c").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_analysis_level_deep() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.c"), "int main(void) { return 0; }\n").unwrap();
    let output = Command::new(bin())
        .args(["--analysis-level", "deep", dir.path().join("a.c").to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_malloc_no_free() {
    let input = "void f() {\n  void* p = malloc(100);\n  return;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_calloc_no_free() {
    let input = "void f() {\n  int* arr = calloc(10, sizeof(int));\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_realloc_no_free() {
    let input = "void f() {\n  buf = realloc(buf, 200);\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_malloc_with_free_ok() {
    let input = "void f() {\n  void* p = malloc(100);\n  free(p);\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_alloc_free_equal_count_ok() {
    let input = "void f() {\n  void* a = malloc(10);\n  void* b = malloc(20);\n  free(a);\n  free(b);\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_if_zero_dead_branch() {
    let input = "if (0) {\n  dead_code();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_if_false_dead_branch() {
    let input = "if (false) {\n  dead_code();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_if_true_no_dead_branch() {
    let input = "if (1) {\n  live_code();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_if_var_no_dead_branch() {
    let input = "if (x) {\n  code();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_dead_branch_comment_ignored() {
    let input = "// if (0) { dead(); }\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_suspicious_cast() {
    let input = "void* ptr = NULL;\nint x = (int)ptr;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-suspicious-cast"));
}

#[test]
fn test_suspicious_cast_with_pointer_decl() {
    let input = "void* p = NULL;\nint v = (int)p;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-suspicious-cast"));
}

#[test]
fn test_normal_cast_ok() {
    let input = "int x = (int)val;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-suspicious-cast"));
}

#[test]
fn test_suspicious_cast_comment_ignored() {
    let input = "// int x = (int)ptr;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-suspicious-cast"));
}

#[test]
fn test_empty_source_strict() {
    let input = "";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.is_empty());
}

#[test]
fn test_resource_leak_comment_ignored() {
    let input = "// void* p = malloc(100);\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_cast_non_ptr_substring_ok() {
    let input = "int fprintf_count = 0;\nint x = (int)fprintf_count;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-suspicious-cast"));
}

#[test]
fn test_else_if_zero_dead_branch() {
    let input = "if (x) {\n  a();\n} else if (0) {\n  dead();\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_hash_if_zero_dead_branch() {
    let input = "#if 0\n  dead();\n#endif\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-dead-branch"));
}

#[test]
fn test_malloc_returned_ok() {
    let input = "void* f() {\n  void* p = malloc(100);\n  return p;\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}

#[test]
fn test_resource_leak_per_function() {
    let input = "void f() {\n  void* p = malloc(100);\n}\nvoid g() {\n  free(p);\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    let leak_count = diags.iter().filter(|d| d.rule_id == "bugprone-resource-leak").count();
    assert_eq!(leak_count, 1);
}

#[test]
fn test_strip_comment_string_literal() {
    let input = "void f() {\n  void* p = malloc(strlen(\"//comment\"));\n}\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let diags = strict::check(&src, &AnalysisConfig::default());
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-resource-leak"));
}
