use std::process::Command;

use vsc_blame::reporter::Reporter;

#[test]
fn test_cli_builds() {
    let output = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("failed to run cargo build");
    assert!(output.status.success(), "cargo build failed");
}

#[test]
fn test_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("failed to run cargo run --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vsc-blame"));
    assert!(stdout.contains("blame"));
    assert!(stdout.contains("traceback"));
    assert!(stdout.contains("diff"));
    assert!(stdout.contains("--vcs"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--no-color"));
}

#[test]
fn test_blame_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "blame", "--help"])
        .output()
        .expect("failed to run cargo run blame --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FILE"));
    assert!(stdout.contains("--all"));
    assert!(stdout.contains("--summary"));
}

#[test]
fn test_traceback_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "traceback", "--help"])
        .output()
        .expect("failed to run cargo run traceback --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("-f"));
    assert!(stdout.contains("--stdin"));
}

#[test]
fn test_diff_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "diff", "--help"])
        .output()
        .expect("failed to run cargo run diff --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--base"));
    assert!(stdout.contains("--head"));
    assert!(stdout.contains("--base-rev"));
    assert!(stdout.contains("--head-rev"));
}

#[test]
fn test_version_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("failed to run cargo run --version");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vsc-blame"));
}

#[test]
fn test_python_traceback_parser() {
    let input = std::fs::read_to_string("tests/fixtures/traceback/sample_py.txt")
        .expect("fixture not found");
    let frames = vsc_blame::parser::traceback_py::parse_python_traceback(&input);
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].file, "example.py");
    assert_eq!(frames[0].line, 42);
    assert_eq!(frames[1].file, "example.py");
    assert_eq!(frames[1].line, 10);
    assert_eq!(frames[2].file, "utils.py");
    assert_eq!(frames[2].line, 5);
}

#[test]
fn test_cpp_stacktrace_parser() {
    let input = std::fs::read_to_string("tests/fixtures/traceback/sample_cpp.txt")
        .expect("fixture not found");
    let frames = vsc_blame::parser::traceback_cpp::parse_cpp_stacktrace(&input);
    assert_eq!(frames.len(), 3);
    assert_eq!(frames[0].file, "crash.c");
    assert_eq!(frames[0].line, 100);
    assert_eq!(frames[1].file, "handler.cpp");
    assert_eq!(frames[1].line, 55);
    assert_eq!(frames[2].file, "main.cpp");
    assert_eq!(frames[2].line, 20);
}

#[test]
fn test_diff_parser() {
    let input = std::fs::read_to_string("tests/fixtures/diff/sample.diff")
        .expect("fixture not found");
    let diffs = vsc_blame::parser::diff::parse_unified_diff(&input).unwrap();
    assert_eq!(diffs.len(), 1);
    assert_eq!(diffs[0].file, "src/main.py");
    assert_eq!(diffs[0].hunks.len(), 2);

    let hunk1 = &diffs[0].hunks[0];
    assert_eq!(hunk1.added_lines.len(), 3);

    let hunk2 = &diffs[0].hunks[1];
    assert_eq!(hunk2.added_lines.len(), 1);
}

#[test]
fn test_blame_aggregation() {
    use std::collections::HashMap;

    let entries = vec![
        vsc_blame::blame::BlameEntry {
            file: "a.py".into(),
            line: 1,
            author: "alice".into(),
            author_mail: "a@a.com".into(),
            author_time: chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            vcs: vsc_blame::blame::VcsKind::Git,
            commit_id: "a".repeat(40),
            summary: "init".into(),
            content: "".into(),
        },
        vsc_blame::blame::BlameEntry {
            file: "a.py".into(),
            line: 2,
            author: "bob".into(),
            author_mail: "b@b.com".into(),
            author_time: chrono::NaiveDate::from_ymd_opt(2024, 6, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            vcs: vsc_blame::blame::VcsKind::Git,
            commit_id: "b".repeat(40),
            summary: "update".into(),
            content: "".into(),
        },
    ];

    let result = vsc_blame::blame::aggregate(entries, &HashMap::new());
    assert_eq!(result.summary.len(), 2);
    assert!(result.suggested_responsible.is_some());
}

#[test]
fn test_text_reporter() {
    let result = vsc_blame::blame::BlameResult {
        entries: vec![vsc_blame::blame::BlameEntry {
            file: "test.py".into(),
            line: 10,
            author: "alice".into(),
            author_mail: "a@a.com".into(),
            author_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
            vcs: vsc_blame::blame::VcsKind::Git,
            commit_id: "abcd1234abcd".into(),
            summary: "test".into(),
            content: "code".into(),
        }],
        summary: vec![],
        suggested_responsible: None,
        uncommitted_lines: vec![],
    };

    let reporter = vsc_blame::reporter::text::TextReporter { no_color: true };
    let mut buf = Vec::new();
    reporter.render(&result, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("File: test.py"));
    assert!(output.contains("alice"));
}

#[test]
fn test_json_reporter() {
    let result = vsc_blame::blame::BlameResult {
        entries: vec![vsc_blame::blame::BlameEntry {
            file: "test.py".into(),
            line: 10,
            author: "alice".into(),
            author_mail: "a@a.com".into(),
            author_time: chrono::DateTime::from_timestamp(1710460800, 0).map(|dt| dt.naive_utc()).unwrap(),
            vcs: vsc_blame::blame::VcsKind::Git,
            commit_id: "abcd1234abcd".into(),
            summary: "test".into(),
            content: "code".into(),
        }],
        summary: vec![],
        suggested_responsible: Some("alice".into()),
        uncommitted_lines: vec![],
    };

    let reporter = vsc_blame::reporter::json::JsonReporter;
    let mut buf = Vec::new();
    reporter.render(&result, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["entries"][0]["author"], "alice");
    assert_eq!(parsed["suggested_responsible"], "alice");
}
