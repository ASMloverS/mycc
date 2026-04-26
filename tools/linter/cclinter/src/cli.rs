use clap::Parser;
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};
use std::path::PathBuf;

use crate::config::{load_config, AnalysisLevel};

type FormatResult = Result<(PathBuf, String, String, Vec<crate::common::diag::Diagnostic>), String>;

#[derive(Parser, Debug)]
#[command(name = "cclinter", version, about = "C language linter")]
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
    pub analysis_level: Option<AnalysisLevel>,

    #[arg(short, long, value_parser = parse_jobs)]
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
    if let Some(ref level) = args.analysis_level {
        config.analysis.level = level.clone();
    }

    if args.verbose {
        eprintln!("config: {config:?}");
    }

    let ignore = build_ignore_matcher(&args);
    let files = collect_files(&args.paths, &ignore)?;

    if files.is_empty() {
        return Ok(());
    }

    if let Some(jobs) = args.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build_global()
            .ok();
    }

    let config_ref = &config.format;
    let results: Vec<FormatResult> = files
        .par_iter()
        .map(|file_path| {
            let mut source = crate::common::source::SourceFile::load(file_path)
                .map_err(|e| e.to_string())?;
            let diags = crate::formatter::format_source(&mut source, config_ref)
                .map_err(|e| e.to_string())?;
            Ok((file_path.clone(), source.original, source.content, diags))
        })
        .collect();

    let mut exit_code = 0u8;

    for result in &results {
        let (file_path, original, formatted, diags) = match result {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error processing file: {e}");
                exit_code |= 1;
                continue;
            }
        };

        if args.verbose && !diags.is_empty() {
            for d in diags {
                eprintln!("{d}");
            }
        }

        if args.check {
            if original != formatted {
                eprintln!("{}: formatting issues found", file_path.display());
                exit_code |= 1;
            }
        } else if args.diff {
            print_diff(original, formatted, file_path);
        } else if args.in_place {
            if original != formatted {
                std::fs::write(file_path, formatted)?;
                if !args.quiet {
                    eprintln!("formatted {}", file_path.display());
                }
            }
        } else {
            print!("{formatted}");
            if !formatted.ends_with('\n') {
                println!();
            }
        }
    }

    if exit_code != 0 {
        std::process::exit(exit_code as i32);
    }
    Ok(())
}

fn parse_jobs(s: &str) -> Result<usize, String> {
    let val: usize = s.parse().map_err(|_| format!("`{s}` is not a positive integer"))?;
    if val == 0 {
        return Err("jobs must be at least 1".into());
    }
    Ok(val)
}

fn build_ignore_matcher(args: &Args) -> crate::ignore::IgnoreMatcher {
    let mut patterns: Vec<String> = args.exclude.clone();

    let ignore_path = std::path::Path::new(".cclinterignore");
    if ignore_path.exists() {
        if let Ok(content) = std::fs::read_to_string(ignore_path) {
            let file_patterns = content
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
                .map(|l| l.to_string());
            patterns.extend(file_patterns);
        }
    }

    crate::ignore::IgnoreMatcher::from_patterns(&patterns)
}

fn collect_files(
    paths: &[PathBuf],
    ignore: &crate::ignore::IgnoreMatcher,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if !ignore.is_ignored(path) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path) {
                let entry = entry?;
                let p = entry.path();
                if let Some(ext) = p.extension() {
                    if (ext == "c" || ext == "h") && !ignore.is_ignored(p) {
                        files.push(p.to_path_buf());
                    }
                }
            }
        } else {
            return Err(format!("path not found: {}", path.display()).into());
        }
    }
    Ok(files)
}

fn print_diff(old: &str, new: &str, path: &std::path::Path) {
    if old == new {
        return;
    }
    println!("--- {}", path.display());
    println!("+++ {}", path.display());
    let diff = TextDiff::from_lines(old, new);
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{}{}", sign, change);
    }
}
