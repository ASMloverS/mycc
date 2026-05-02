use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "pylinter", version, about = "Python 3.12+ linter: format + style check + static analysis")]
pub struct Args {
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(short, long, conflicts_with = "check", conflicts_with = "diff")]
    pub in_place: bool,

    #[arg(long, conflicts_with = "diff", conflicts_with = "in_place")]
    pub check: bool,

    #[arg(long, conflicts_with = "check", conflicts_with = "in_place")]
    pub diff: bool,

    #[arg(long)]
    pub format_only: bool,

    #[arg(long, value_enum)]
    pub analysis_level: Option<crate::config::AnalysisLevel>,

    #[arg(short, long, value_parser = parse_jobs)]
    pub jobs: Option<usize>,

    #[arg(long)]
    pub exclude: Vec<String>,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short = 'v', long)]
    pub verbose: bool,
}

fn parse_jobs(s: &str) -> Result<usize, String> {
    let val: usize = s.parse().map_err(|_| format!("`{s}` is not a positive integer"))?;
    if val == 0 {
        return Err("jobs must be at least 1".into());
    }
    Ok(val)
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let _args = Args::parse();
    Ok(())
}
