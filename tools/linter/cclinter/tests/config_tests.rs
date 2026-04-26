mod common;

use cclinter::config::{
    load_config, AnalysisConfig, AnalysisLevel, Config,
};
use std::path::PathBuf;

#[test]
fn default_config_has_expected_values() {
    let config = Config::default();
    assert_eq!(config.format.column_limit, 120);
    assert_eq!(config.format.indent_width, 2);
    assert!(!config.format.use_tabs);
    assert_eq!(config.analysis.level, AnalysisLevel::Basic);
    assert_eq!(config.check.complexity.max_function_lines, 100);
}

#[test]
fn load_config_from_valid_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.yaml");
    std::fs::write(
        &path,
        "format:\n  column_limit: 80\n  indent_width: 4\n",
    )
    .unwrap();
    let config = load_config(Some(&path)).unwrap();
    assert_eq!(config.format.column_limit, 80);
    assert_eq!(config.format.indent_width, 4);
}

#[test]
fn load_config_missing_file_returns_error() {
    let path = PathBuf::from("/nonexistent/path/.cclinter.yaml");
    let result = load_config(Some(&path));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("config file not found"),
        "expected 'config file not found', got: {msg}"
    );
}

#[test]
fn load_config_malformed_yaml_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.yaml");
    std::fs::write(&path, "format:\n  column_limit: not_a_number\n").unwrap();
    let result = load_config(Some(&path));
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("invalid config"),
        "expected 'invalid config', got: {msg}"
    );
}

#[test]
fn partial_yaml_gets_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("partial.yaml");
    std::fs::write(&path, "format:\n  column_limit: 80\n").unwrap();
    let config = load_config(Some(&path)).unwrap();
    assert_eq!(config.format.column_limit, 80);
    assert_eq!(config.format.indent_width, 2);
    assert_eq!(config.analysis.level, AnalysisLevel::Basic);
}

#[test]
fn find_config_in_cwd() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join(".cclinter.yaml");
    std::fs::write(
        &config_path,
        "format:\n  column_limit: 99\n",
    )
    .unwrap();
    let orig = std::env::current_dir().unwrap();
    let result = std::env::set_current_dir(dir.path());
    if result.is_err() {
        std::env::set_current_dir(&orig).ok();
        return;
    }
    let config = load_config(None).unwrap();
    std::env::set_current_dir(&orig).unwrap();
    assert_eq!(config.format.column_limit, 99);
}

#[test]
fn empty_yaml_produces_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.yaml");
    std::fs::write(&path, "").unwrap();
    let config = load_config(Some(&path)).unwrap();
    assert_eq!(config, Config::default());
}

#[test]
fn include_sorting_none_deserialization() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sort.yaml");
    std::fs::write(&path, "format:\n  include_sorting: none\n").unwrap();
    let config = load_config(Some(&path)).unwrap();
    assert_eq!(
        config.format.include_sorting,
        cclinter::config::IncludeSorting::Disabled
    );
}

#[test]
fn analysis_level_default_is_basic() {
    let config = AnalysisConfig::default();
    assert_eq!(config.level, AnalysisLevel::Basic);
}

#[test]
fn disabled_include_sorting_serializes_as_none() {
    use cclinter::config::IncludeSorting;
    let yaml = serde_yaml::to_string(&IncludeSorting::Disabled).unwrap();
    assert!(
        yaml.contains("none"),
        "expected 'none' in yaml, got:\n{yaml}"
    );
}
