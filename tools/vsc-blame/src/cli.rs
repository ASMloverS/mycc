use std::collections::HashMap;

use clap::{Parser, Subcommand};

use crate::blame::{self, BlameResult, LineSpec, VcsKind};
use crate::config::Config;
use crate::util::AppError;

#[derive(Parser)]
#[command(name = "vsc-blame", version, about = "Blame tool: git/svn blame with traceback/diff parsing and smart aggregation")]
struct Cli {
    #[arg(long, value_name = "VCS")]
    vcs: Option<String>,

    #[arg(long, value_name = "FORMAT")]
    format: Option<String>,

    #[arg(long, value_name = "PATH")]
    config: Option<String>,

    #[arg(long)]
    no_color: bool,

    #[arg(short, long)]
    quiet: bool,

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Blame specified file/lines")]
    Blame {
        file: String,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        summary: bool,
    },

    #[command(about = "Parse traceback/stack trace and blame each frame")]
    Traceback {
        text: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
        #[arg(long)]
        stdin: bool,
    },

    #[command(about = "Parse diff and blame changed lines")]
    Diff {
        #[arg(short, long, conflicts_with = "stdin")]
        file: Option<String>,
        #[arg(long, conflicts_with = "file")]
        stdin: bool,
        #[arg(long, conflicts_with_all = ["base_rev", "head_rev", "file", "stdin"])]
        base: Option<String>,
        #[arg(long, conflicts_with_all = ["base_rev", "head_rev", "file", "stdin"])]
        head: Option<String>,
        #[arg(long, conflicts_with_all = ["base", "head", "file", "stdin"])]
        base_rev: Option<String>,
        #[arg(long, conflicts_with_all = ["base", "head", "file", "stdin"])]
        head_rev: Option<String>,
    },
}

pub fn run(raw_args: &[String]) -> i32 {
    let args = preprocess_args(raw_args);
    let cli = match Cli::try_parse_from(&args) {
        Ok(c) => c,
        Err(e) => {
            e.print().ok();
            return if e.exit_code() == 2 { 2 } else { 1 };
        }
    };

    match execute(cli) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("{}", e);
            e.code
        }
    }
}

fn preprocess_args(args: &[String]) -> Vec<String> {
    if args.len() <= 1 {
        return args.to_vec();
    }

    let subs = ["blame", "traceback", "diff", "help"];
    let flag_with_value = ["--vcs", "--format", "--config", "-f"];

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--" {
            break;
        }
        if arg.starts_with('-') {
            let is_flag_with_value = flag_with_value
                .iter()
                .any(|f| arg == f || arg.starts_with(&format!("{}=", f)));
            let has_embedded_value = arg.contains('=');
            if is_flag_with_value && !has_embedded_value && i + 1 < args.len() && !args[i + 1].starts_with('-') {
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if subs.contains(&arg.as_str()) {
            return args.to_vec();
        }
        let mut result = args[..i].to_vec();
        result.push("blame".to_string());
        result.extend_from_slice(&args[i..]);
        return result;
    }

    args.to_vec()
}

fn execute(cli: Cli) -> Result<(), AppError> {
    let config_path = Config::resolve_config_path(cli.config.as_deref());
    let config = Config::load(&config_path).map_err(AppError::error)?;

    let no_color = config.resolve_no_color(cli.no_color);
    if no_color {
        colored::control::set_override(false);
    }

    let quiet = cli.quiet;
    let verbose = cli.verbose;

    let vcs_kind = resolve_vcs(cli.vcs.as_deref(), &config, quiet)?;

    let backend = match vcs_kind {
        VcsKind::Git => Box::new(crate::vcs::git::GitBackend) as Box<dyn crate::vcs::VcsBackend>,
        VcsKind::Svn => Box::new(crate::vcs::svn::SvnBackend) as Box<dyn crate::vcs::VcsBackend>,
    };

    let format = config.resolve_format(cli.format.as_deref());
    let output_path = config.resolve_output(None);
    let aliases = config.author_aliases;

    let result = match cli.command {
        Commands::Blame { file, all, summary } => {
            do_blame(&*backend, &file, all, summary, verbose, &aliases)?
        }
        Commands::Traceback { text, file, stdin } => {
            do_traceback(&*backend, text.as_deref(), file.as_deref(), stdin, verbose, &aliases, quiet)?
        }
        Commands::Diff { file, stdin, base, head, base_rev, head_rev } => {
            do_diff(&*backend, file.as_deref(), stdin, base.as_deref(), head.as_deref(), base_rev.as_deref(), head_rev.as_deref(), vcs_kind, verbose, &aliases, quiet)?
        }
    };

    if result.entries.is_empty() && result.uncommitted_lines.is_empty() {
        return Err(AppError::empty("no blame results"));
    }

    let reporter = crate::reporter::get_reporter(&format, no_color)?;

    match output_path {
        Some(path) => {
            let mut f = std::fs::File::create(&path).map_err(|e| {
                AppError::error(format!("cannot create output file {}: {}", path, e))
            })?;
            reporter.render(&result, &mut f)?;
        }
        None => {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            reporter.render(&result, &mut out)?;
        }
    }

    Ok(())
}

fn resolve_vcs(cli: Option<&str>, config: &Config, _quiet: bool) -> Result<VcsKind, AppError> {
    let vcs_str = config.resolve_vcs(cli);

    match vcs_str.as_deref() {
        Some("git") => Ok(VcsKind::Git),
        Some("svn") => Ok(VcsKind::Svn),
        Some("auto") | None => crate::vcs::detect::detect_vcs(),
        Some(other) => Err(AppError::usage(format!("unknown VCS: {}", other))),
    }
}

fn do_blame(
    backend: &dyn crate::vcs::VcsBackend,
    file_spec: &str,
    all: bool,
    summary: bool,
    verbose: bool,
    aliases: &HashMap<String, Vec<String>>,
) -> Result<BlameResult, AppError> {
    let (file, lines) = blame::parse_file_spec(file_spec)?;

    let lines = if all { LineSpec::All } else { lines };

    if verbose {
        eprintln!("[INFO] blaming {} with {}", file, backend.name());
    }

    let entries = backend.blame_file(&file, &lines)?;
    let result = blame::aggregate(entries, aliases);

    if summary {
        Ok(BlameResult {
            entries: vec![],
            summary: result.summary,
            suggested_responsible: result.suggested_responsible,
            uncommitted_lines: result.uncommitted_lines,
        })
    } else {
        Ok(result)
    }
}

fn do_traceback(
    backend: &dyn crate::vcs::VcsBackend,
    text: Option<&str>,
    file: Option<&str>,
    use_stdin: bool,
    verbose: bool,
    aliases: &HashMap<String, Vec<String>>,
    quiet: bool,
) -> Result<BlameResult, AppError> {
    let input = crate::util::read_input(text, file, use_stdin, "text")?;

    let py_frames = crate::parser::traceback_py::parse_python_traceback(&input);
    let cpp_frames = crate::parser::traceback_cpp::parse_cpp_stacktrace(&input);

    let frames: Vec<(String, usize)> = if !py_frames.is_empty() {
        py_frames.into_iter().map(|f| (f.file, f.line)).collect()
    } else if !cpp_frames.is_empty() {
        cpp_frames.into_iter().map(|f| (f.file, f.line)).collect()
    } else {
        return Err(AppError::usage("no traceback or stack trace frames found in input"));
    };

    if verbose {
        eprintln!("[INFO] found {} frames", frames.len());
    }

    let mut grouped: HashMap<String, Vec<(usize, usize)>> = HashMap::new();
    for (file, line) in &frames {
        let entry = grouped.entry(file.clone()).or_default();
        entry.push((*line, *line));
    }

    let mut all_entries = Vec::new();
    for (file, segs) in grouped {
        let spec = LineSpec::Multi(segs);
        if verbose {
            eprintln!("[INFO] blaming {} with {}", file, backend.name());
        }
        match backend.blame_file(&file, &spec) {
            Ok(entries) => all_entries.extend(entries),
            Err(e) => {
                crate::util::warn(
                    &format!("failed to blame {}: {}", file, e.message),
                    quiet,
                );
            }
        }
    }

    Ok(blame::aggregate(all_entries, aliases))
}

#[allow(clippy::too_many_arguments)]
fn do_diff(
    backend: &dyn crate::vcs::VcsBackend,
    file: Option<&str>,
    use_stdin: bool,
    base: Option<&str>,
    head: Option<&str>,
    base_rev: Option<&str>,
    head_rev: Option<&str>,
    vcs_kind: VcsKind,
    verbose: bool,
    aliases: &HashMap<String, Vec<String>>,
    quiet: bool,
) -> Result<BlameResult, AppError> {
    let diffs = if file.is_some() || use_stdin {
        let input = crate::util::read_input(None, file, use_stdin, "diff")?;
        crate::parser::diff::parse_unified_diff(&input)?
    } else if base.is_some() || head.is_some() {
        if vcs_kind != VcsKind::Git {
            return Err(AppError::usage("--base/--head requires --vcs git"));
        }
        let base_val = base.unwrap_or("HEAD~1");
        let head_val = head.unwrap_or("HEAD");
        if verbose {
            eprintln!("[INFO] running diff {}..{}", base_val, head_val);
        }
        backend.diff_revisions(base_val, head_val)?
    } else if base_rev.is_some() || head_rev.is_some() {
        if vcs_kind != VcsKind::Svn {
            return Err(AppError::usage("--base-rev/--head-rev requires --vcs svn"));
        }
        let base_val = base_rev.unwrap_or("BASE");
        let head_val = head_rev.unwrap_or("HEAD");
        if verbose {
            eprintln!("[INFO] running svn diff -r {}:{}", base_val, head_val);
        }
        backend.diff_revisions(base_val, head_val)?
    } else {
        return Err(AppError::usage(
            "specify --base/--head, --base-rev/--head-rev, -f <file>, or --stdin",
        ));
    };

    if verbose {
        eprintln!("[INFO] found {} files in diff", diffs.len());
    }

    let mut all_entries = Vec::new();
    for diff in &diffs {
        if diff.hunks.is_empty() {
            continue;
        }
        let mut all_added: Vec<usize> = diff
            .hunks
            .iter()
            .flat_map(|h| h.added_lines.iter().copied())
            .collect();
        all_added.sort();
        all_added.dedup();

        if all_added.is_empty() {
            continue;
        }

        let segs: Vec<(usize, usize)> = all_added.into_iter().map(|l| (l, l)).collect();
        let spec = LineSpec::Multi(segs);

        if verbose {
            eprintln!("[INFO] blaming {} with {}", diff.file, backend.name());
        }

        match backend.blame_file(&diff.file, &spec) {
            Ok(entries) => all_entries.extend(entries),
            Err(e) => {
                crate::util::warn(
                    &format!("failed to blame {}: {}", diff.file, e.message),
                    quiet,
                );
            }
        }
    }

    Ok(blame::aggregate(all_entries, aliases))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocess_no_subcommand() {
        let args = vec![
            "vsc-blame".to_string(),
            "foo.py:10".to_string(),
        ];
        let processed = preprocess_args(&args);
        assert_eq!(processed[1], "blame");
        assert_eq!(processed[2], "foo.py:10");
    }

    #[test]
    fn test_preprocess_with_global_flag() {
        let args = vec![
            "vsc-blame".to_string(),
            "--vcs".to_string(),
            "git".to_string(),
            "foo.py:10".to_string(),
        ];
        let processed = preprocess_args(&args);
        assert_eq!(processed[1], "--vcs");
        assert_eq!(processed[2], "git");
        assert_eq!(processed[3], "blame");
        assert_eq!(processed[4], "foo.py:10");
    }

    #[test]
    fn test_preprocess_with_subcommand() {
        let args = vec![
            "vsc-blame".to_string(),
            "blame".to_string(),
            "foo.py:10".to_string(),
        ];
        let processed = preprocess_args(&args);
        assert_eq!(processed, args);
    }

    #[test]
    fn test_preprocess_traceback() {
        let args = vec![
            "vsc-blame".to_string(),
            "traceback".to_string(),
            "-f".to_string(),
            "trace.txt".to_string(),
        ];
        let processed = preprocess_args(&args);
        assert_eq!(processed, args);
    }

    #[test]
    fn test_preprocess_no_args() {
        let args = vec!["vsc-blame".to_string()];
        let processed = preprocess_args(&args);
        assert_eq!(processed.len(), 1);
    }

    #[test]
    fn test_resolve_vcs_explicit() {
        let config = Config::default();
        assert_eq!(resolve_vcs(Some("git"), &config, true).unwrap(), VcsKind::Git);
        assert_eq!(resolve_vcs(Some("svn"), &config, true).unwrap(), VcsKind::Svn);
    }

    #[test]
    fn test_resolve_vcs_invalid() {
        let config = Config::default();
        assert!(resolve_vcs(Some("mercurial"), &config, true).is_err());
    }

    #[test]
    fn test_cli_parse_blame() {
        let args = vec![
            "vsc-blame".to_string(),
            "blame".to_string(),
            "foo.py:10".to_string(),
        ];
        let cli = Cli::try_parse_from(&args).unwrap();
        match cli.command {
            Commands::Blame { file, .. } => assert_eq!(file, "foo.py:10"),
            _ => panic!("expected Blame command"),
        }
    }

    #[test]
    fn test_cli_parse_traceback() {
        let args = vec![
            "vsc-blame".to_string(),
            "traceback".to_string(),
            "-f".to_string(),
            "trace.txt".to_string(),
        ];
        let cli = Cli::try_parse_from(&args).unwrap();
        match cli.command {
            Commands::Traceback { file, .. } => assert_eq!(file.unwrap(), "trace.txt"),
            _ => panic!("expected Traceback command"),
        }
    }

    #[test]
    fn test_cli_parse_diff_flags() {
        let args = vec![
            "vsc-blame".to_string(),
            "diff".to_string(),
            "--base".to_string(),
            "HEAD~3".to_string(),
        ];
        let cli = Cli::try_parse_from(&args).unwrap();
        match cli.command {
            Commands::Diff { base, .. } => assert_eq!(base.unwrap(), "HEAD~3"),
            _ => panic!("expected Diff command"),
        }
    }
}
