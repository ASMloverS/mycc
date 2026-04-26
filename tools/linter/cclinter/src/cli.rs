use clap::Parser;
use std::path::PathBuf;

use crate::config::{load_config, AnalysisLevel};

#[derive(Parser, Debug)]
#[command(name = "cclinter", version, about = "C language linter")]
pub struct Args {
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(short, long)]
    pub in_place: bool,

    #[arg(long)]
    pub check: bool,

    #[arg(long)]
    pub diff: bool,

    #[arg(long)]
    pub format_only: bool,

    #[arg(long, value_enum)]
    pub analysis_level: Option<AnalysisLevel>,

    #[arg(short, long)]
    pub jobs: Option<usize>,

    #[arg(long)]
    pub exclude: Vec<String>,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short = 'v', long)]
    pub verbose: bool,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut config = load_config(args.config.as_ref())?;
    if let Some(level) = args.analysis_level {
        config.analysis.level = level;
    }

    if args.verbose {
        eprintln!("config: {config:?}");
    }

    Ok(())
}
