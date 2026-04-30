mod common;

use std::fs;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    common::get_bin()
}

fn fixtures_dir() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures");
    p
}

#[test]
fn test_full_pipeline_format_check() {
    let full_test = fixtures_dir().join("input/full_test.c");
    let output = Command::new(bin())
        .args(["--check", "--format-only", full_test.to_str().unwrap()])
        .output()
        .unwrap();
    assert_ne!(
        output.status.code(),
        Some(0),
        "full_test.c has format issues, should exit non-zero"
    );
}

#[test]
fn test_analysis_deep_flags_issues() {
    let analysis_test = fixtures_dir().join("analysis_test.c");
    let output = Command::new(bin())
        .args([
            "--check",
            "--analysis-level",
            "deep",
            analysis_test.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let has_null_deref = stderr.contains("null-deref");
    let has_buffer_overflow = stderr.contains("buffer-overflow");
    let has_resource_leak = stderr.contains("resource-leak");
    assert!(
        has_null_deref || has_buffer_overflow || has_resource_leak,
        "deep analysis should flag at least one issue, stderr: {stderr:?}"
    );
}

#[test]
fn test_analysis_none_exits_0() {
    let clean = fixtures_dir().join("input/clean.c");
    let output = Command::new(bin())
        .args([
            "--check",
            "--analysis-level",
            "none",
            "--format-only",
            clean.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "analysis none + format-only on clean file should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_in_place_creates_formatted() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.c");
    fs::write(&file_path, "int x=1;\n").unwrap();

    let output = Command::new(bin())
        .args(["-i", "--format-only", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(
        content.contains("int x = 1;"),
        "spacing should be fixed, got: {content:?}"
    );
}

#[test]
fn test_diff_mode_output() {
    let dirty = fixtures_dir().join("input/dirty.c");
    let output = Command::new(bin())
        .args(["--diff", "--format-only", dirty.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "diff mode should exit 0, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "diff should produce output");
}

#[test]
fn test_exit_code_combines_format_and_analysis() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("combined.c");
    // Format issue (tab), analysis issue (NULL deref at deep level)
    fs::write(&file_path, "int* p = NULL;\n\t*p = 42;\n").unwrap();

    let output = Command::new(bin())
        .args([
            "--check",
            "--analysis-level",
            "deep",
            file_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    let has_format = code & 1 != 0;
    let has_analysis = code & 4 != 0;
    assert!(
        has_format && has_analysis,
        "exit code should have bit 1 (format={has_format}) and bit 4 (analysis={has_analysis}), got code={code}, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_analysis_deep_flags_resource_leak() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("leak.c");
    fs::write(&file, "#include <stdlib.h>\nvoid f() { void* p = malloc(10); }\n").unwrap();
    let output = Command::new(bin())
        .args(["--check", "--analysis-level", "deep", file.to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("resource-leak"),
        "should flag resource leak, got: {stderr:?}"
    );
    assert_eq!(
        output.status.code().unwrap() & 4,
        4,
        "exit code should have bit 4 set"
    );
}

#[test]
fn test_format_only_skips_analysis() {
    let analysis_test = fixtures_dir().join("analysis_test.c");
    let output = Command::new(bin())
        .args(["--check", "--format-only", analysis_test.to_str().unwrap()])
        .output()
        .unwrap();
    let code = output.status.code().unwrap();
    assert_eq!(
        code & 4,
        0,
        "--format-only should not trigger analysis (exit code bit 4), got code={code}, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
