use std::path::PathBuf;

use pylinter::checker::check_source;
use pylinter::common::source::SourceFile;
use pylinter::config::CheckConfig;

fn check(src: &str, path: &str) -> Vec<pylinter::common::diag::Diagnostic> {
    let source = SourceFile::from_string(src, PathBuf::from(path));
    check_source(&source, &CheckConfig::default())
}

fn rule_ids(diags: &[pylinter::common::diag::Diagnostic]) -> Vec<&str> {
    diags.iter().map(|d| d.rule_id.as_str()).collect()
}

fn assert_has_rule(ids: &[&str], rule: &str) {
    assert!(ids.iter().any(|r| *r == rule), "expected {rule} diagnostic, got: {ids:?}");
}

#[test]
fn all_checkers_run_on_fixture() {
    let input = std::fs::read_to_string(
        std::path::Path::new(file!())
            .parent()
            .unwrap()
            .join("fixtures/input/checker_test.py"),
    )
    .unwrap();
    let diags = check(&input, "checker_test.py");
    let ids = rule_ids(&diags);

    assert_has_rule(&ids, "readability-naming-function");
    assert_has_rule(&ids, "readability-naming-class");
    assert_has_rule(&ids, "readability-naming-variable");
    assert_has_rule(&ids, "readability-unused-import");
    assert_has_rule(&ids, "readability-magic-number");
    assert_has_rule(&ids, "readability-prohibited");
    assert_has_rule(&ids, "readability-missing-function-docstring");
    assert_has_rule(&ids, "readability-missing-class-docstring");
}

#[test]
fn clean_code_no_diagnostics() {
    let src = "\"\"\"Module doc.\"\"\"\nimport os\nprint(os.path)\n\n\ndef my_func():\n    \"\"\"Doc.\"\"\"\n    pass\n\n\nclass MyClass:\n    \"\"\"Doc.\"\"\"\n    pass\n";
    let diags = check(src, "test.py");
    let ids = rule_ids(&diags);
    assert!(ids.is_empty(), "expected no diagnostics, got: {ids:?}");
}

#[test]
fn check_pipeline_returns_empty_for_parse_error() {
    let diags = check("def foo(:\n", "test.py");
    assert!(diags.is_empty());
}

#[test]
fn check_pipeline_wires_all_six_checkers() {
    // Trigger each checker individually and verify the rule_id prefix
    let naming_src = "def BadFunc():\n    pass\n";
    let diags = check(naming_src, "test.py");
    assert!(diags.iter().any(|d| d.rule_id.starts_with("readability-naming-")));

    let complexity_src = {
        let mut s = "def foo():\n".to_string();
        for i in 0..55 {
            s.push_str(&format!("    x = {}\n", i));
        }
        s
    };
    let diags = check(&complexity_src, "test.py");
    assert!(diags.iter().any(|d| d.rule_id == "readability-function-size"));

    let magic_src = "x = calculate(42)\n";
    let diags = check(magic_src, "test.py");
    assert!(diags.iter().any(|d| d.rule_id == "readability-magic-number"));

    let unused_src = "import os\nx = 1\n";
    let diags = check(unused_src, "test.py");
    assert!(diags.iter().any(|d| d.rule_id == "readability-unused-import"));

    let prohibited_src = "x = eval('1+1')\n";
    let diags = check(prohibited_src, "test.py");
    assert!(diags.iter().any(|d| d.rule_id == "readability-prohibited"));

    let docstring_src = "x = 1\n";
    let diags = check(docstring_src, "test.py");
    assert!(diags.iter().any(|d| d.rule_id == "readability-missing-module-docstring"));
}

#[test]
fn check_source_with_config_disabling_checkers() {
    let src = "x = calculate(42)\n";
    let source = SourceFile::from_string(src, PathBuf::from("test.py"));
    let mut config = CheckConfig::default();
    config.magic_number.enabled = false;
    config.docstring.module = false;
    config.docstring.class = false;
    config.docstring.function = false;
    config.unused_import.enabled = false;
    let diags = check_source(&source, &config);
    assert!(diags.is_empty());
}

#[test]
fn checker_diagnostics_have_correct_fields() {
    let src = "def BadFunc():\n    pass\n";
    let diags = check(src, "test.py");
    let diag = diags.iter().find(|d| d.rule_id == "readability-naming-function").unwrap();
    assert_eq!(diag.file, "test.py");
    assert!(diag.line > 0);
    assert!(diag.col > 0);
    assert!(diag.source_line.is_some());
    assert!(diag.message.contains("BadFunc"));
}
