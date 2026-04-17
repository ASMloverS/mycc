use std::process::Command;

#[test]
fn binary_exists_and_runs() {
    let bin = assert_cmd::get_bin();
    assert!(bin.exists(), "binary not found at {:?}", bin);
    let output = Command::new(&bin).arg("--help").output().unwrap();
    assert!(output.status.success());
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
