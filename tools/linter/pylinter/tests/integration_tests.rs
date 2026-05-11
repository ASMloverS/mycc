use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn pylinter_bin() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from("target/debug/pylinter");
    path.set_extension(std::env::consts::EXE_EXTENSION);
    path
}

fn bin_exists() -> bool {
    pylinter_bin().exists()
}

fn write_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn idempotent_full_pipeline() {
    if !bin_exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", pylinter_bin().display());
        return;
    }
    let dir = TempDir::new().unwrap();
    let src = "import os\nimport unused_module\n\ndef foo(x=[]):\n    try:\n        pass\n    except:\n        pass\n";
    write_file(dir.path(), "test.py", src);

    let run = |label: &str| -> String {
        let output = std::process::Command::new(pylinter_bin())
            .arg("--quiet")
            .arg("--analysis-level")
            .arg("deep")
            .arg(dir.path().join("test.py"))
            .output()
            .unwrap_or_else(|e| panic!("failed to run pylinter ({label}): {e}"));
        String::from_utf8_lossy(&output.stderr).to_string()
    };

    let stderr1 = run("first");
    let stderr2 = run("second");
    assert_eq!(stderr1, stderr2, "pipeline output should be idempotent");
}

#[test]
fn check_mode_returns_nonzero() {
    if !bin_exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", pylinter_bin().display());
        return;
    }
    let dir = TempDir::new().unwrap();
    let src = "import os\nimport unused_module\n\ndef foo(x=[]):\n    try:\n        pass\n    except:\n        pass\n";
    write_file(dir.path(), "messy.py", src);

    let output = std::process::Command::new(pylinter_bin())
        .arg("--check")
        .arg("--analysis-level")
        .arg("basic")
        .arg(dir.path().join("messy.py"))
        .output()
        .expect("failed to run pylinter");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let code = output.status.code().unwrap_or(0);
    assert_ne!(code, 0, "should exit non-zero, stderr: {stderr}");
    assert!(
        code & 4 != 0,
        "exit code should have analyzer bit (4) set, got code={code}, stderr: {stderr}"
    );
}

#[test]
fn clean_file_returns_zero() {
    if !bin_exists() {
        eprintln!("skipping: {} not found, run `cargo build` first", pylinter_bin().display());
        return;
    }
    let dir = TempDir::new().unwrap();
    let src = "\"\"\"Module doc.\"\"\"\nimport os\n\n\nprint(os.path)\n\n\ndef my_func():\n    \"\"\"Doc.\"\"\"\n    return os.path\n\n\nclass MyClass:\n    \"\"\"Doc.\"\"\"\n\n    def method(self):\n        \"\"\"Doc.\"\"\"\n        return self\n";
    write_file(dir.path(), "clean.py", src);

    let output = std::process::Command::new(pylinter_bin())
        .arg("--check")
        .arg("--analysis-level")
        .arg("deep")
        .arg(dir.path().join("clean.py"))
        .output()
        .expect("failed to run pylinter");

    let code = output.status.code().unwrap_or(-1);
    assert_eq!(code, 0, "clean file should exit 0, stderr: {}", String::from_utf8_lossy(&output.stderr));
}
