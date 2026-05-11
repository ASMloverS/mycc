use std::path::PathBuf;

use pylinter::analyzer::analyze_source;
use pylinter::common::source::SourceFile;
use pylinter::config::{AnalysisConfig, AnalysisLevel};

fn analyze(src: &str, level: AnalysisLevel) -> Vec<pylinter::common::diag::Diagnostic> {
    let source = SourceFile::from_string(src, PathBuf::from("test.py"));
    let config = AnalysisConfig { level };
    analyze_source(&source, &config.level, &config)
}

fn rule_ids(diags: &[pylinter::common::diag::Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.rule_id.as_str()).collect()
}

fn assert_has_rule(ids: &[&str], rule: &str) {
    assert!(
        ids.iter().any(|r| *r == rule),
        "expected {rule} diagnostic, got: {ids:?}"
    );
}

// ── Per-level tests ──────────────────────────────────────────────────────────

#[test]
fn basic_level_catches_mutable_default() {
    let diags = analyze("def foo(x=[]):\n    pass\n", AnalysisLevel::Basic);
    assert_has_rule(&rule_ids(&diags), "bugprone-mutable-default");
}

#[test]
fn basic_level_catches_bare_except() {
    let diags = analyze("try:\n    pass\nexcept:\n    pass\n", AnalysisLevel::Basic);
    assert_has_rule(&rule_ids(&diags), "bugprone-bare-except");
}

#[test]
fn strict_level_catches_unnecessary_pass() {
    let diags = analyze(
        "def foo():\n    \"\"\"Doc.\"\"\"\n    pass\n",
        AnalysisLevel::Strict,
    );
    assert_has_rule(&rule_ids(&diags), "readability-unnecessary-pass");
}

#[test]
fn deep_level_catches_unused_variable() {
    let diags = analyze(
        "def foo():\n    x = 1\n    return 2\n",
        AnalysisLevel::Deep,
    );
    assert_has_rule(&rule_ids(&diags), "deadcode-unused-variable");
}

#[test]
fn deep_level_catches_shadow_builtin() {
    let diags = analyze(
        "def foo(list):\n    pass\n",
        AnalysisLevel::Deep,
    );
    assert_has_rule(&rule_ids(&diags), "bugprone-shadow-builtin");
}

#[test]
fn level_none_no_diagnostics() {
    let src = "def foo(x=[]):\n    try:\n        pass\n    except:\n        pass\n";
    let diags = analyze(src, AnalysisLevel::None);
    assert!(diags.is_empty(), "expected no diagnostics for None level, got: {:?}", rule_ids(&diags));
}

// ── Tier inclusion tests ─────────────────────────────────────────────────────

#[test]
fn basic_includes_basic_rules_only() {
    let src = "def foo(x=[]):\n    \"\"\"Doc.\"\"\"\n    pass\n";
    let diags = analyze(src, AnalysisLevel::Basic);
    let ids = rule_ids(&diags);
    assert_has_rule(&ids, "bugprone-mutable-default");
    assert!(
        !ids.iter().any(|r| *r == "readability-unnecessary-pass"),
        "basic level should not include strict rules, got: {ids:?}"
    );
    assert!(
        !ids.iter().any(|r| *r == "deadcode-unused-variable"),
        "basic level should not include deep rules, got: {ids:?}"
    );
}

#[test]
fn strict_includes_basic_rules() {
    let src = "def foo(x=[]):\n    \"\"\"Doc.\"\"\"\n    pass\n";
    let diags = analyze(src, AnalysisLevel::Strict);
    let ids = rule_ids(&diags);
    assert_has_rule(&ids, "bugprone-mutable-default");
    assert_has_rule(&ids, "readability-unnecessary-pass");
    assert!(
        !ids.iter().any(|r| *r == "deadcode-unused-variable"),
        "strict level should not include deep rules, got: {ids:?}"
    );
}

#[test]
fn deep_includes_all_rules() {
    let src = "def foo(x=[]):\n    \"\"\"Doc.\"\"\"\n    pass\n    y = 1\n    return 2\n";
    let diags = analyze(src, AnalysisLevel::Deep);
    let ids = rule_ids(&diags);
    assert_has_rule(&ids, "bugprone-mutable-default");
    assert_has_rule(&ids, "readability-unnecessary-pass");
    assert_has_rule(&ids, "deadcode-unused-variable");
}

// ── Full pipeline E2E ────────────────────────────────────────────────────────

#[test]
fn full_pipeline_end_to_end() {
    let input = std::fs::read_to_string(
        std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join("fixtures/input/full_analysis.py"),
    )
    .unwrap();

    let diags = analyze(&input, AnalysisLevel::Basic);
    let ids = rule_ids(&diags);
    assert_has_rule(&ids, "bugprone-mutable-default");
    assert_has_rule(&ids, "bugprone-bare-except");
    assert_has_rule(&ids, "bugprone-none-comparison");
    assert_has_rule(&ids, "bugprone-missing-self");

    let deep_diags = analyze(&input, AnalysisLevel::Deep);
    let deep_ids = rule_ids(&deep_diags);
    assert_has_rule(&deep_ids, "bugprone-mutable-default");
    assert_has_rule(&deep_ids, "bugprone-bare-except");
    assert_has_rule(&deep_ids, "bugprone-shadow-builtin");
    assert_has_rule(&deep_ids, "deadcode-unused-variable");
}
