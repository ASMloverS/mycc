use std::fs;
use std::path::PathBuf;

use pylinter::common::source::SourceFile;
use pylinter::config::FormatConfig;
use pylinter::formatter::format_source;

fn fixtures_dir() -> PathBuf {
    let mut p = PathBuf::from(file!());
    p.pop();
    p.push("fixtures");
    p
}

fn read_fixture(relative: &str) -> String {
    fs::read_to_string(fixtures_dir().join(relative)).unwrap()
}

fn assert_formats_to(input: &str, expected: &str) {
    let mut source = SourceFile::from_string(input, PathBuf::from("test.py"));
    format_source(&mut source, &FormatConfig::default()).unwrap();
    assert_eq!(source.content, expected);
}

#[test]
fn format_dirty_file() {
    assert_formats_to(&read_fixture("input/dirty.py"), &read_fixture("expected/dirty.py"));
}

#[test]
fn idempotent_formatting() {
    let input = read_fixture("input/dirty.py");
    let mut src1 = SourceFile::from_string(&input, PathBuf::from("test.py"));
    format_source(&mut src1, &FormatConfig::default()).unwrap();
    let mut src2 = SourceFile::from_string(&src1.content, PathBuf::from("test.py"));
    format_source(&mut src2, &FormatConfig::default()).unwrap();
    assert_eq!(src1.content, src2.content);
}

#[test]
fn preserves_already_formatted() {
    let formatted = read_fixture("expected/dirty.py");
    assert_formats_to(&formatted, &formatted);
}

#[test]
fn empty_file_stays_empty() {
    assert_formats_to("", "");
}

#[test]
fn encoding_normalization_in_pipeline() {
    assert_formats_to(
        "\u{feff}x = 1\r\ndef foo():\r\n\tpass\r\n",
        "x = 1\n\n\ndef foo():\n    pass\n",
    );
}

#[test]
fn trailing_whitespace_removed() {
    assert_formats_to("x = 1   \ny = 2\t\n", "x = 1\ny = 2\n");
}

#[test]
fn tab_indent_normalized() {
    assert_formats_to("if True:\n\tpass\n", "if True:\n    pass\n");
}

#[test]
fn blank_lines_normalized() {
    assert_formats_to("x = 1\n\n\n\n\n\ny = 2\n", "x = 1\n\n\ny = 2\n");
}

#[test]
fn all_modules_combined() {
    assert_formats_to(
        "\u{feff}import os   \r\nx = 1\r\ndef foo():\r\n\tpass\r\nclass Bar:\r\n  def baz(self):\r\n    pass\r\n",
        "import os\nx = 1\n\n\ndef foo():\n    pass\n\n\nclass Bar:\n\n    def baz(self):\n        pass\n",
    );
}

#[test]
fn invalid_syntax_passes_through() {
    let input = "= invalid\n";
    assert_formats_to(input, input);
}
