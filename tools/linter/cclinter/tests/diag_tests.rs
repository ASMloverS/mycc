use cclinter::common::diag::{Diagnostic, Severity};

fn disable_color() {
    colored::control::set_override(false);
}

#[test]
fn test_diag_format_warning() {
    disable_color();
    let d = Diagnostic::new(
        "foo.c".into(),
        10,
        5,
        Severity::Warning,
        "unused-var",
        "Variable 'x' is unused",
    );
    let s = format!("{d}");
    assert!(s.contains("foo.c:10:5:"), "should contain location, got: {s}");
    assert!(s.contains("warning"), "should contain severity, got: {s}");
    assert!(s.contains("[unused-var]"), "should contain rule id, got: {s}");
    assert!(
        s.contains("Variable 'x' is unused"),
        "should contain message, got: {s}"
    );
}

#[test]
fn test_diag_format_error() {
    disable_color();
    let d = Diagnostic::new(
        "bar.c".into(),
        1,
        1,
        Severity::Error,
        "prohibited-fn",
        "Use of gets() is prohibited",
    );
    let s = format!("{d}");
    assert!(s.contains("error"), "should contain error, got: {s}");
    assert!(s.contains("[prohibited-fn]"), "should contain rule id, got: {s}");
}

#[test]
fn test_diag_format_note() {
    disable_color();
    let d = Diagnostic::new(
        "baz.c".into(),
        5,
        1,
        Severity::Note,
        "info",
        "Informational note",
    );
    let s = format!("{d}");
    assert!(s.contains("note"), "should contain note, got: {s}");
}

#[test]
fn test_diag_with_source_line() {
    disable_color();
    let d = Diagnostic::new_with_source(
        "test.c".into(),
        3,
        10,
        Severity::Warning,
        "magic-number",
        "Magic number 42",
        "  int x = 42;",
    );
    let s = format!("{d}");
    assert!(
        s.contains("int x = 42;"),
        "should contain source line, got: {s}"
    );
    assert!(s.contains("magic-number"), "should contain rule id, got: {s}");
}

#[test]
fn test_diag_without_source_line() {
    disable_color();
    let d = Diagnostic::new(
        "test.c".into(),
        1,
        1,
        Severity::Warning,
        "test",
        "no source",
    );
    let s = format!("{d}");
    assert!(
        !s.contains('\n'),
        "should not contain newline without source, got: {s:?}"
    );
}
