mod common;

use std::process::Command;

#[test]
fn binary_exists_and_runs() {
    let bin = common::get_bin();
    assert!(bin.exists(), "binary not found at {:?}", bin);
    let output = Command::new(&bin).arg("--help").output().unwrap();
    assert!(output.status.success());
}
