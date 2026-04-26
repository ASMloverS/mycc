mod common;

use cclinter::ignore::IgnoreMatcher;
use std::path::Path;

#[test]
fn test_ignore_pattern() {
    let patterns = vec!["*.generated.c".to_string(), "build/".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("foo.generated.c")));
    assert!(matcher.is_ignored(Path::new("build/main.c")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_ignore_comment_lines() {
    let content = "# comment\n*.bak\n\nvendor/\n";
    let matcher = IgnoreMatcher::from_string(content);
    assert!(matcher.is_ignored(Path::new("test.c.bak")));
    assert!(matcher.is_ignored(Path::new("vendor/foo.c")));
}

#[test]
fn test_empty_matcher() {
    let matcher = IgnoreMatcher::from_string("");
    assert!(!matcher.is_ignored(Path::new("anything.c")));
}

#[test]
fn test_ignore_path_separator_patterns() {
    let patterns = vec!["src/generated/".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("src/generated/parser.c")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_ignore_multiple_patterns_same_file() {
    let patterns = vec![
        "*.generated.c".to_string(),
        "build/".to_string(),
        "*.bak".to_string(),
    ];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("test.generated.c")));
    assert!(matcher.is_ignored(Path::new("build/test.c")));
    assert!(matcher.is_ignored(Path::new("test.c.bak")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_ignore_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let ignore_path = dir.path().join(".cclinterignore");
    std::fs::write(&ignore_path, "# ignore generated\n*.generated.c\nvendor/\n").unwrap();

    let matcher = IgnoreMatcher::from_file(&ignore_path);
    assert!(matcher.is_ignored(Path::new("foo.generated.c")));
    assert!(matcher.is_ignored(Path::new("vendor/bar.c")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_ignore_missing_file_is_empty() {
    let matcher = IgnoreMatcher::from_file(Path::new("/nonexistent/.cclinterignore"));
    assert!(!matcher.is_ignored(Path::new("anything.c")));
}

#[test]
fn test_doublestar_patterns() {
    let patterns = vec!["**/generated/**".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("generated/foo.c")));
    assert!(matcher.is_ignored(Path::new("src/generated/bar.c")));
    assert!(matcher.is_ignored(Path::new("a/b/generated/c/d.c")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_wildcard_matches_at_depth() {
    let patterns = vec!["*.o".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("foo.o")));
    assert!(matcher.is_ignored(Path::new("build/foo.o")));
    assert!(!matcher.is_ignored(Path::new("foo.c")));
}

#[test]
fn test_plain_name_matches_directory_contents() {
    let patterns = vec!["build".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("build/foo.c")));
    assert!(matcher.is_ignored(Path::new("src/build/bar.c")));
    assert!(!matcher.is_ignored(Path::new("src/main.c")));
}

#[test]
fn test_leading_slash_root_only() {
    let patterns = vec!["/build".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(matcher.is_ignored(Path::new("build/foo.c")));
    assert!(!matcher.is_ignored(Path::new("src/build/foo.c")));
}

#[test]
fn test_negation_pattern_skipped() {
    let patterns = vec!["!foo.c".to_string(), "*.bak".to_string()];
    let matcher = IgnoreMatcher::from_patterns(&patterns);
    assert!(!matcher.is_ignored(Path::new("foo.c")));
    assert!(matcher.is_ignored(Path::new("test.bak")));
}
