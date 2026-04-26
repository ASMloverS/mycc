use cclinter::checker::include_guard::check_include_guard;
use cclinter::common::source::SourceFile;
use std::path::PathBuf;

fn disable_color() {
    colored::control::set_override(false);
}

#[test]
fn test_missing_include_guard() {
    disable_color();
    let input = "int x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_has_pragma_once() {
    disable_color();
    let input = "#pragma once\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_has_ifndef_guard() {
    disable_color();
    let input = "#ifndef HEADER_H\n#define HEADER_H\nint x;\n#endif\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let mut config = cclinter::config::IncludeGuardConfig::default();
    config.style = cclinter::config::IncludeGuardStyle::Ifndef;
    let diags = check_include_guard(&src, &config);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_duplicate_include() {
    disable_color();
    let input = "#include <stdio.h>\n#include <stdlib.h>\n#include <stdio.h>\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-duplicate-include"));
}

#[test]
fn test_no_guard_for_c_file() {
    disable_color();
    let input = "int x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_no_duplicate_for_unique_includes() {
    disable_color();
    let input = "#include <stdio.h>\n#include <stdlib.h>\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-duplicate-include"));
}

#[test]
fn test_hpp_header_needs_guard() {
    disable_color();
    let input = "int x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.hpp"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_empty_header_no_warning() {
    disable_color();
    let src = SourceFile::from_string("", PathBuf::from("header.h"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    assert!(!diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_ifndef_at_line_200_not_guard() {
    disable_color();
    let input = "int x;\n".repeat(200) + "#ifndef GUARD_H\n#define GUARD_H\n#endif\n";
    let src = SourceFile::from_string(&input, PathBuf::from("header.h"));
    let mut config = cclinter::config::IncludeGuardConfig::default();
    config.style = cclinter::config::IncludeGuardStyle::Ifndef;
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_triple_duplicate() {
    disable_color();
    let input = "#include <stdio.h>\n#include <stdio.h>\n#include <stdio.h>\n";
    let src = SourceFile::from_string(input, PathBuf::from("test.c"));
    let config = cclinter::config::IncludeGuardConfig::default();
    let diags = check_include_guard(&src, &config);
    let dup_count = diags.iter().filter(|d| d.rule_id == "bugprone-duplicate-include").count();
    assert_eq!(dup_count, 2);
}

#[test]
fn test_ifdef_not_guard() {
    disable_color();
    let input = "#ifdef DEBUG\nint x;\n#endif\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let mut config = cclinter::config::IncludeGuardConfig::default();
    config.style = cclinter::config::IncludeGuardStyle::Ifndef;
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_ifndef_style_rejects_pragma() {
    disable_color();
    let input = "#pragma once\nint x;\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let mut config = cclinter::config::IncludeGuardConfig::default();
    config.style = cclinter::config::IncludeGuardStyle::Ifndef;
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}

#[test]
fn test_pragma_style_rejects_ifndef() {
    disable_color();
    let input = "#ifndef HEADER_H\n#define HEADER_H\nint x;\n#endif\n";
    let src = SourceFile::from_string(input, PathBuf::from("header.h"));
    let mut config = cclinter::config::IncludeGuardConfig::default();
    config.style = cclinter::config::IncludeGuardStyle::PragmaOnce;
    let diags = check_include_guard(&src, &config);
    assert!(diags.iter().any(|d| d.rule_id == "bugprone-missing-include-guard"));
}
