use cclinter::analyzer::analyze_source;
use cclinter::common::source::SourceFile;
use cclinter::config::{AnalysisConfig, AnalysisLevel};
use std::path::PathBuf;

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

mod common;

use std::process::Command;

fn bin() -> std::path::PathBuf {
    common::get_bin()
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
