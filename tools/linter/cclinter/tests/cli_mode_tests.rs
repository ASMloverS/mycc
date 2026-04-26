mod common;

use std::process::Command;
use std::fs;

fn bin() -> std::path::PathBuf {
    common::get_bin()
}

fn fixtures_dir() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures");
    p
}

#[test]
fn test_default_mode_prints_formatted() {
    let dirty = fixtures_dir().join("input/dirty.c");
    let output = Command::new(bin())
        .arg(&dirty)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
    assert!(
        !stdout.contains('\t'),
        "should not contain tabs, got: {stdout:?}"
    );
}

#[test]
fn test_check_mode_exits_1_on_issues() {
    let dirty = fixtures_dir().join("input/dirty.c");
    let output = Command::new(bin())
        .args(["--check", dirty.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("formatting issues"),
        "stderr should mention formatting issues, got: {stderr:?}"
    );
}

#[test]
fn test_check_mode_exits_0_on_clean() {
    let clean = fixtures_dir().join("input/clean.c");
    let output = Command::new(bin())
        .args(["--check", clean.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_diff_mode_shows_changes() {
    let dirty = fixtures_dir().join("input/dirty.c");
    let output = Command::new(bin())
        .args(["--diff", dirty.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_minus = stdout.lines().any(|l| l.starts_with('-'));
    let has_plus = stdout.lines().any(|l| l.starts_with('+'));
    assert!(
        has_minus && has_plus,
        "should have - and + lines, got: {stdout:?}"
    );
}

#[test]
fn test_diff_mode_no_changes_on_clean() {
    let clean = fixtures_dir().join("input/clean.c");
    let output = Command::new(bin())
        .args(["--diff", clean.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_changes = stdout
        .lines()
        .filter(|l| l.starts_with('-') && !l.starts_with("--- "))
        .any(|_| true)
        || stdout
            .lines()
            .filter(|l| l.starts_with('+') && !l.starts_with("+++ "))
            .any(|_| true);
    assert!(!has_changes, "should have no change lines, got: {stdout:?}");
}

#[test]
fn test_in_place_modifies_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.c");
    fs::write(&file_path, b"int x;\t\n").unwrap();

    let output = Command::new(bin())
        .args(["-i", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "int x;\n", "trailing tab should be stripped");
}

#[test]
fn test_in_place_no_change_on_clean() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.c");
    let clean = "int x;\n";
    fs::write(&file_path, clean).unwrap();

    let output = Command::new(bin())
        .args(["-i", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), clean);
}

#[test]
fn test_quiet_mode_suppresses_output() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.c");
    fs::write(&file_path, b"int x;\t\n").unwrap();

    let output = Command::new(bin())
        .args(["-i", "--quiet", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("formatted"));
}

#[test]
fn test_exclude_ignores_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_a = dir.path().join("a.c");
    let file_b = dir.path().join("b.c");
    fs::write(&file_a, b"int x;\t\n").unwrap();
    fs::write(&file_b, b"int x;\t\n").unwrap();

    let output = Command::new(bin())
        .args([
            "--check",
            "--exclude",
            "a.c",
            dir.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("b.c") && !stderr.contains("a.c"),
        "should mention b.c not a.c, got: {stderr:?}"
    );
}

#[test]
fn test_directory_argument_walks_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.c"), "int x;\n").unwrap();
    fs::write(dir.path().join("b.c"), "int y;\n").unwrap();
    fs::write(dir.path().join("ignore.txt"), "not a c file\n").unwrap();

    let output = Command::new(bin())
        .args(["--check", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_verbose_prints_config() {
    let clean = fixtures_dir().join("input/clean.c");
    let output = Command::new(bin())
        .args(["--verbose", clean.to_str().unwrap()])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("config:"),
        "verbose should print config, got: {stderr:?}"
    );
}

#[test]
fn test_cclinterignore_excludes_files() {
    let dir = tempfile::tempdir().unwrap();
    let ignore_path = dir.path().join(".cclinterignore");
    fs::write(&ignore_path, "a.c\n").unwrap();
    let file_a = dir.path().join("a.c");
    let file_b = dir.path().join("b.c");
    fs::write(&file_a, "int main(){\nreturn 0;\n}\n").unwrap();
    fs::write(&file_b, "int x;\t\n").unwrap();

    let output = Command::new(bin())
        .args(["--check", dir.path().to_str().unwrap()])
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("a.c"),
        "a.c should be ignored, got: {stderr:?}"
    );
    assert!(
        stderr.contains("b.c"),
        "b.c should be checked, got: {stderr:?}"
    );
}

#[test]
fn test_nonexistent_path_errors() {
    let output = Command::new(bin())
        .args(["--check", "nonexistent_file_xyz.c"])
        .output()
        .unwrap();
    assert_ne!(output.status.code(), Some(0));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("path not found"),
        "should report missing path, got: {stderr:?}"
    );
}

#[test]
fn test_conflicting_flags_rejected() {
    let output = Command::new(bin())
        .args(["--check", "--diff", "tests/fixtures/input/clean.c"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cannot be used with") || stderr.contains("conflict"),
        "should report conflict, got: {stderr:?}"
    );
}

#[test]
fn test_jobs_flag_processes_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.c"), b"int x;\t\n").unwrap();
    fs::write(dir.path().join("b.c"), b"int y;\t\n").unwrap();

    let output = Command::new(bin())
        .args(["--check", "-j2", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("a.c"), "should mention a.c, got: {stderr:?}");
    assert!(stderr.contains("b.c"), "should mention b.c, got: {stderr:?}");
}

#[test]
fn test_jobs_1_is_serial() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.c"), "int x;\n").unwrap();

    let output = Command::new(bin())
        .args(["--check", "-j1", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_parallel_default_mode_stdout() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.c"), b"int x;\t\n").unwrap();
    fs::write(dir.path().join("b.c"), b"int y;\t\n").unwrap();

    let output = Command::new(bin())
        .args(["-j2", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains('\t'), "tabs should be removed, got: {stdout:?}");
}

#[test]
fn test_jobs_zero_rejected() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("a.c"), "int x;\n").unwrap();

    let output = Command::new(bin())
        .args(["-j0", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "-j0 should be rejected"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value") || stderr.contains("at least 1"),
        "should explain -j0 rejection, got: {stderr:?}"
    );
}
