use std::process::Command;

#[test]
fn help_flag_works() {
    let bin = assert_cmd::get_bin();
    let output = Command::new(&bin).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cclinter"));
    assert!(stdout.contains("--check"));
    assert!(stdout.contains("--diff"));
    assert!(stdout.contains("--in-place"));
    assert!(stdout.contains("--format-only"));
    assert!(stdout.contains("--analysis-level"));
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--exclude"));
    assert!(stdout.contains("--jobs"));
}

mod assert_cmd {
    use std::path::PathBuf;

    pub fn get_bin() -> PathBuf {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.pop();
        path.push("cclinter");
        path.set_extension(std::env::consts::EXE_EXTENSION);
        path
    }
}
