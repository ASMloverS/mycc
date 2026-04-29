use cclinter::analyzer::{analyze_source, basic};
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
