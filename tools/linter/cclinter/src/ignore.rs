use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;

pub struct IgnoreMatcher {
    set: GlobSet,
}

fn expand_patterns(pat: &str) -> Vec<String> {
    let mut results = Vec::new();

    if pat.starts_with('!') {
        eprintln!("warning: negation patterns not supported, skipping: {pat}");
        return results;
    }

    let (pat, root_only) = match pat.strip_prefix('/') {
        Some(s) => (s, true),
        None => (pat, false),
    };

    let has_trailing_slash = pat.ends_with('/');
    let pat = pat.trim_end_matches('/');

    if pat.is_empty() {
        return results;
    }

    let has_glob = pat.chars().any(|c| matches!(c, '*' | '?' | '['));
    let has_sep = pat.contains('/');

    if root_only {
        results.push(format!("{pat}/**"));
    } else if !has_glob && !has_sep {
        results.push(format!("**/{pat}/**"));
    } else if !has_glob || has_trailing_slash {
        results.push(format!("{pat}/**"));
    } else {
        results.push(pat.to_string());
    }

    results
}

impl IgnoreMatcher {
    pub fn from_patterns(patterns: &[String]) -> Self {
        let mut builder = GlobSetBuilder::new();
        for pat in patterns {
            for expanded in expand_patterns(pat) {
                if let Ok(glob) = Glob::new(&expanded) {
                    let _ = builder.add(glob);
                }
            }
        }
        let set = builder.build().unwrap_or_else(|_| GlobSet::empty());
        Self { set }
    }

    pub fn from_string(content: &str) -> Self {
        let patterns: Vec<String> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| l.to_string())
            .collect();
        Self::from_patterns(&patterns)
    }

    pub fn from_file(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => Self::from_string(&content),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::from_patterns(&[]),
            Err(e) => {
                eprintln!("warning: cannot read {}: {e}", path.display());
                Self::from_patterns(&[])
            }
        }
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.set.is_match(path_str.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }
}
