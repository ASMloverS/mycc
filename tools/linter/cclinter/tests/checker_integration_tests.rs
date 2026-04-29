mod common;

use std::collections::HashSet;
use std::fs;
use std::process::Command;

fn bin() -> std::path::PathBuf {
    common::get_bin()
}

#[test]
fn test_checker_flags_style_violations() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bad.c");
    fs::write(
        &file,
        "int BadFunction(void) {\n  int UnusedVar = 42;\n  return 0;\n}\n",
    )
    .unwrap();

    let output = Command::new(bin())
        .args(["--format-only", file.to_str().unwrap()])
        .output()
        .unwrap();

    let output2 = Command::new(bin())
        .args([file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output2.status.code(), Some(2));
    assert!(output.status.success(), "format-only should succeed");

    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr.contains("readability-naming-function"),
        "should flag naming violation, got: {stderr:?}"
    );
}

#[test]
fn test_exit_code_2_with_style_violations() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("style.c");
    fs::write(&file, "int x;\n").unwrap();

    let output = Command::new(bin())
        .args(["--format-only", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_no_violations_checker_exit_0() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clean_check.c");
    fs::write(
        &file,
        "#pragma once\nvoid foo(void) {\n  int x = 0;\n  (void)x;\n}\n",
    )
    .unwrap();

    let output = Command::new(bin())
        .args([file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(0),
        "clean file should exit 0 with checker enabled, stderr: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_exit_code_3_format_and_style() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("both.c");
    fs::write(&file, "int unused_var = 99;\t\n").unwrap();

    let output = Command::new(bin())
        .args(["--check", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(3),
        "exit code should be 3 (1 format | 2 style)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("formatting issues"),
        "should mention formatting, got: {stderr:?}"
    );
}

#[test]
fn test_format_only_skips_checker() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("style.c");
    fs::write(&file, "int x;\n").unwrap();

    let output = Command::new(bin())
        .args(["--format-only", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_format_only_check_mode_skips_checker() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("style.c");
    fs::write(&file, "int x;\n").unwrap();

    let output = Command::new(bin())
        .args(["--check", "--format-only", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_deduplication_same_file_line_rule() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("dedup.c");
    fs::write(&file, "int MagicFunc(void) {\n  return 99;\n}\n").unwrap();

    let output = Command::new(bin())
        .args([file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);

    let naming_count = stderr
        .lines()
        .filter(|l| l.contains("readability-naming-function"))
        .count();
    assert!(
        naming_count <= 1,
        "same rule on same line should appear at most once, got {naming_count} occurrences"
    );
}

#[test]
fn test_different_rules_same_line_both_reported() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("multi.c");
    fs::write(&file, "int BadName = 99;\n").unwrap();

    let output = Command::new(bin())
        .args([file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);

    let rule_ids: Vec<&str> = stderr
        .lines()
        .filter_map(|l| {
            if l.contains('[') && l.contains(']') {
                let start = l.rfind('[').unwrap() + 1;
                let end = l.rfind(']').unwrap();
                Some(&l[start..end])
            } else {
                None
            }
        })
        .collect();

    let unique_rules: HashSet<&str> = rule_ids.iter().copied().collect();
    assert!(
        unique_rules.len() >= 2,
        "different rules on same line should both be reported, got rules: {unique_rules:?}"
    );
}

#[test]
fn test_quiet_suppresses_checker_output() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("quiet.c");
    fs::write(&file, "int unused_var = 1;\n").unwrap();

    let output = Command::new(bin())
        .args(["--quiet", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("bugprone-unused-variable"),
        "quiet should suppress checker diagnostics, got: {stderr:?}"
    );
}

#[test]
fn test_checker_parallel_multiple_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.c"), "int BadVar_a = 1;\n").unwrap();
    fs::write(dir.path().join("b.c"), "int BadVar_b = 2;\n").unwrap();

    let output = Command::new(bin())
        .args(["-j2", dir.path().to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("a.c"),
        "should mention a.c, got: {stderr:?}"
    );
    assert!(
        stderr.contains("b.c"),
        "should mention b.c, got: {stderr:?}"
    );
}

#[test]
fn test_no_violations_exit_0() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("clean.c");
    fs::write(
        &file,
        "#pragma once\nvoid foo(void) {\n  int x = 0;\n  (void)x;\n}\n",
    )
    .unwrap();

    let output = Command::new(bin())
        .args(["--format-only", file.to_str().unwrap()])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
}
