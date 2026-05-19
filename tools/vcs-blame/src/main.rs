use vcs_blame::cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let code = cli::run(&args);
    std::process::exit(code);
}
