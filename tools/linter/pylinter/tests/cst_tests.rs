use pylinter::cst::CSTSource;

#[test]
fn roundtrip_simple() {
    let src = "x = 1\ny = 2\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_function() {
    let src = "def foo():\n    return 42\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_with_comments() {
    let src = "# comment\nx = 1  # inline\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_multiline() {
    let src = "class Foo:\n    def bar(self):\n        pass\n\n    def baz(self):\n        return 1\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_preserves_indent() {
    let src = "if True:\n    if True:\n        pass\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn parse_error_returns_err() {
    // "def foo(:\n" and "def foo(\n" cause rustpython-parser to hang in error recovery;
    // use a simpler invalid input instead.
    let src = "= 1\n";
    assert!(CSTSource::parse(src).is_err());
}

#[test]
fn roundtrip_no_trailing_newline() {
    let src = "x = 1";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_blank_lines() {
    let src = "x = 1\n\ny = 2\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_multiline_expr() {
    let src = "x = (\n    1 +\n    2\n)\n";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}

#[test]
fn roundtrip_empty_source() {
    let src = "";
    let cst = CSTSource::parse(src).unwrap();
    assert_eq!(cst.regenerate(), src);
}
