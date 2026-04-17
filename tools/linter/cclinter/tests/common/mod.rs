use std::path::PathBuf;

pub fn get_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("cclinter");
    path.set_extension(std::env::consts::EXE_EXTENSION);
    path
}
