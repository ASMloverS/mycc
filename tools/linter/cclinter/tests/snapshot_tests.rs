use cclinter::common::source::SourceFile;
use cclinter::config::FormatConfig;
use cclinter::formatter::format_source;
use std::path::PathBuf;

fn run_snapshot(input_name: &str) {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let input_path = base.join("tests/fixtures/input").join(input_name);
    let expected_path = base.join("tests/fixtures/expected").join(input_name);

    let input = std::fs::read_to_string(&input_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", input_path.display(), e));
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", expected_path.display(), e));

    let mut source = SourceFile::from_string(&input, input_path);
    format_source(&mut source, &FormatConfig::default())
        .expect("format_source should not fail");

    let expected_normalized = expected.replace("\r\n", "\n");
    let result_normalized = source.content.replace("\r\n", "\n");
    assert_eq!(
        result_normalized, expected_normalized,
        "\n--- INPUT ---\n{}\n--- EXPECTED ---\n{}\n--- GOT ---\n{}",
        input, expected_normalized, result_normalized
    );
}

#[test]
fn test_full_snapshot() {
    run_snapshot("full_test.c");
}

#[test]
fn test_encoding_snapshot() {
    run_snapshot("encoding_test.c");
}

#[test]
fn test_comment_snapshot() {
    run_snapshot("comment_test.c");
}

#[test]
fn test_crlf_conversion() {
    let input = "int x = 1;\r\nint y = 2;\r\n";
    let mut source =
        SourceFile::from_string(input, PathBuf::from("crlf_test.c"));
    format_source(&mut source, &FormatConfig::default())
        .expect("format should not fail");
    assert!(
        !source.content.contains("\r\n"),
        "CRLF should be converted to LF"
    );
    assert_eq!(source.content, "int x = 1;\nint y = 2;\n");
}
