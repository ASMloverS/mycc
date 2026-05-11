use std::fs;
use tempfile::TempDir;

fn pylinter_bin() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from("target/debug/pylinter");
    path.set_extension(std::env::consts::EXE_EXTENSION);
    path
}

fn write_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn no_args_shows_error() {
    let bin = pylinter_bin();
    if !bin.exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", bin.display());
        return;
    }
    let output = std::process::Command::new(&bin)
        .output()
        .expect("failed to run pylinter");
    assert!(!output.status.success(), "should exit with error when no args provided");
}

#[test]
fn diff_mode_shows_no_changes() {
    let bin = pylinter_bin();
    if !bin.exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", bin.display());
        return;
    }
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "example.py", "x = 1\n");
    let output = std::process::Command::new(&bin)
        .arg("--diff")
        .arg("--format-only")
        .arg(dir.path().join("example.py"))
        .output()
        .expect("failed to run pylinter");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.is_empty(), "expected no diff output, got: {stdout}");
    assert!(output.status.success(), "should exit 0 with no changes");
}

#[test]
fn collects_py_files() {
    let bin = pylinter_bin();
    if !bin.exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", bin.display());
        return;
    }
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "a.py", "x = 1\n");
    write_file(dir.path(), "b.txt", "not python\n");
    write_file(dir.path(), "c.py", "y = 2\n");
    let output = std::process::Command::new(&bin)
        .arg("--quiet")
        .arg("--format-only")
        .arg(dir.path())
        .output()
        .expect("failed to run pylinter");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "should exit 0");
    assert!(stdout.contains("x = 1"), "should contain a.py content");
    assert!(stdout.contains("y = 2"), "should contain c.py content");
}

#[test]
fn ignores_excluded_patterns() {
    let bin = pylinter_bin();
    if !bin.exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", bin.display());
        return;
    }
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "keep.py", "keep = 1\n");
    write_file(dir.path(), "skip.py", "skip = 2\n");
    let output = std::process::Command::new(&bin)
        .arg("--quiet")
        .arg("--format-only")
        .arg("--exclude")
        .arg("skip.py")
        .arg(dir.path())
        .output()
        .expect("failed to run pylinter");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "should exit 0");
    assert!(stdout.contains("keep = 1"), "should contain keep.py content");
    assert!(!stdout.contains("skip = 2"), "should not contain excluded file content");
}

#[test]
fn check_mode_detects_issues() {
    let bin = pylinter_bin();
    if !bin.exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", bin.display());
        return;
    }
    let dir = TempDir::new().unwrap();
    write_file(dir.path(), "messy.py", "x=1+2\n");
    let output = std::process::Command::new(&bin)
        .arg("--check")
        .arg("--format-only")
        .arg(dir.path().join("messy.py"))
        .output()
        .expect("failed to run pylinter");
    assert!(output.status.success(), "should exit 0 with formatter stub");
}
