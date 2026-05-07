use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::ComplexityConfig;
use rustpython_parser::ast::{self, Ranged};

pub fn check_complexity(source: &SourceFile, config: &ComplexityConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Ok(ast) =
        rustpython_parser::parse(&source.content, rustpython_parser::Mode::Module, "<input>")
    else {
        return diags;
    };

    let lines = source.lines();
    let mut locator = rustpython_parser::source_code::RandomLocator::new(&source.content);

    if lines.len() > config.max_file_lines {
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            1,
            1,
            Severity::Warning,
            "readability-file-size",
            &format!(
                "File has {} lines (max {})",
                lines.len(),
                config.max_file_lines
            ),
            lines.first().unwrap_or(&""),
        ));
    }

    let body = match &ast {
        ast::Mod::Module(m) => &m.body,
        _ => return diags,
    };

    walk_stmts(
        body,
        &lines,
        source,
        config,
        &mut locator,
        &mut diags,
        0,
    );

    diags
}

fn walk_stmts(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    config: &ComplexityConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
    nesting_depth: usize,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                handle_function(f.start(), f.end(), &f.body, nesting_depth, lines, source, config, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                handle_function(f.start(), f.end(), &f.body, nesting_depth, lines, source, config, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                check_span_length(c.start(), c.end(), config.max_class_lines, "Class", "readability-class-size", lines, source, locator, diags);
                walk_stmts(&c.body, lines, source, config, locator, diags, nesting_depth);
            }
            ast::Stmt::If(i) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    i.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&i.body, lines, source, config, locator, diags, new_depth);
                if i.orelse.len() == 1 && matches!(&i.orelse[0], ast::Stmt::If(_)) {
                    walk_stmts(&i.orelse, lines, source, config, locator, diags, nesting_depth);
                } else {
                    walk_stmts(&i.orelse, lines, source, config, locator, diags, new_depth);
                }
            }
            ast::Stmt::For(f) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    f.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&f.body, lines, source, config, locator, diags, new_depth);
                walk_stmts(&f.orelse, lines, source, config, locator, diags, new_depth);
            }
            ast::Stmt::AsyncFor(f) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    f.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&f.body, lines, source, config, locator, diags, new_depth);
                walk_stmts(&f.orelse, lines, source, config, locator, diags, new_depth);
            }
            ast::Stmt::While(w) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    w.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&w.body, lines, source, config, locator, diags, new_depth);
                walk_stmts(&w.orelse, lines, source, config, locator, diags, new_depth);
            }
            ast::Stmt::Match(m) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    m.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                for case in &m.cases {
                    walk_stmts(&case.body, lines, source, config, locator, diags, new_depth);
                }
            }
            ast::Stmt::With(w) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    w.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&w.body, lines, source, config, locator, diags, new_depth);
            }
            ast::Stmt::AsyncWith(w) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    w.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&w.body, lines, source, config, locator, diags, new_depth);
            }
            ast::Stmt::Try(t) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    t.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&t.body, lines, source, config, locator, diags, new_depth);
                walk_handlers(&t.handlers, lines, source, config, locator, diags, new_depth);
                walk_stmts(&t.orelse, lines, source, config, locator, diags, new_depth);
                walk_stmts(&t.finalbody, lines, source, config, locator, diags, new_depth);
            }
            ast::Stmt::TryStar(t) => {
                let new_depth = nesting_depth + 1;
                check_nesting(
                    t.start(),
                    new_depth,
                    config.max_nesting_depth,
                    lines,
                    source,
                    locator,
                    diags,
                );
                walk_stmts(&t.body, lines, source, config, locator, diags, new_depth);
                walk_handlers(&t.handlers, lines, source, config, locator, diags, new_depth);
                walk_stmts(&t.orelse, lines, source, config, locator, diags, new_depth);
                walk_stmts(&t.finalbody, lines, source, config, locator, diags, new_depth);
            }
            _ => {}
        }
    }
}

fn walk_handlers(
    handlers: &[ast::ExceptHandler],
    lines: &[&str],
    source: &SourceFile,
    config: &ComplexityConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
    nesting_depth: usize,
) {
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        walk_stmts(&h.body, lines, source, config, locator, diags, nesting_depth);
    }
}

fn check_span_length(
    start: ast::TextSize,
    end: ast::TextSize,
    max_lines: usize,
    label: &str,
    rule_id: &str,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    let start_line = locator.locate(start).row.to_usize();
    let end_line = locator.locate(end).row.to_usize();
    let span = end_line - start_line + 1;
    if span > max_lines {
        let source_line = lines.get(start_line - 1).copied().unwrap_or("");
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            start_line,
            1,
            Severity::Warning,
            rule_id,
            &format!("{} spans {} lines (max {})", label, span, max_lines),
            source_line,
        ));
    }
}

fn handle_function(
    start: ast::TextSize,
    end: ast::TextSize,
    body: &[ast::Stmt],
    nesting_depth: usize,
    lines: &[&str],
    source: &SourceFile,
    config: &ComplexityConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    check_span_length(start, end, config.max_function_lines, "Function", "readability-function-size", lines, source, locator, diags);
    let new_depth = nesting_depth + 1;
    check_nesting(start, new_depth, config.max_nesting_depth, lines, source, locator, diags);
    walk_stmts(body, lines, source, config, locator, diags, new_depth);
}

fn check_nesting(
    node_start: ast::TextSize,
    depth: usize,
    max_depth: usize,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    if depth > max_depth {
        let line = locator.locate(node_start).row.to_usize();
        let source_line = lines.get(line - 1).copied().unwrap_or("");
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            line,
            1,
            Severity::Warning,
            "readability-deep-nesting",
            &format!(
                "Nesting depth {} exceeds max {}",
                depth, max_depth
            ),
            source_line,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::source::SourceFile;
    use std::path::PathBuf;

    fn check(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_complexity(&source, &ComplexityConfig::default())
    }

    fn rule_ids(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule_id.as_str()).collect()
    }

    #[test]
    fn function_too_long() {
        let mut src = "def foo():\n".to_string();
        for i in 0..55 {
            src.push_str(&format!("    x = {}\n", i));
        }
        let diags = check(&src);
        assert_eq!(rule_ids(&diags), vec!["readability-function-size"]);
    }

    #[test]
    fn async_function_too_long() {
        let mut src = "async def foo():\n".to_string();
        for i in 0..55 {
            src.push_str(&format!("    x = {}\n", i));
        }
        let diags = check(&src);
        assert_eq!(rule_ids(&diags), vec!["readability-function-size"]);
    }

    #[test]
    fn class_too_long() {
        let mut src = "class Foo:\n".to_string();
        for i in 0..305 {
            src.push_str(&format!("    x = {}\n", i));
        }
        let diags = check(&src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-class-size"));
    }

    #[test]
    fn file_too_long() {
        let mut src = String::new();
        for i in 0..1005 {
            src.push_str(&format!("x = {}\n", i));
        }
        let diags = check(&src);
        assert_eq!(rule_ids(&diags), vec!["readability-file-size"]);
    }

    #[test]
    fn nesting_too_deep() {
        let src = concat!(
            "def foo():\n",
            "    if True:\n",
            "        if True:\n",
            "            if True:\n",
            "                if True:\n",
            "                    if True:\n",
            "                        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn nesting_at_limit_ok() {
        let src = concat!(
            "def foo():\n",
            "    if True:\n",
            "        if True:\n",
            "            if True:\n",
            "                pass\n",
        );
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn no_warning_within_limits() {
        let src = "def foo():\n    pass\n";
        let diags = check(src);
        assert!(diags.is_empty());
    }

    #[test]
    fn function_exactly_at_limit_ok() {
        let mut src = "def foo():\n".to_string();
        for i in 0..49 {
            src.push_str(&format!("    x = {}\n", i));
        }
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.rule_id == "readability-function-size"));
    }

    #[test]
    fn diagnostic_contains_source_line() {
        let mut src = "def foo():\n".to_string();
        for i in 0..55 {
            src.push_str(&format!("    x = {}\n", i));
        }
        let diags = check(&src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].source_line.as_deref(), Some("def foo():"));
    }

    #[test]
    fn multiple_nesting_violations() {
        let src = concat!(
            "def foo():\n",
            "    if True:\n",
            "        if True:\n",
            "            if True:\n",
            "                if True:\n",
            "                    if True:\n",
            "                        pass\n",
            "                    if True:\n",
            "                        pass\n",
        );
        let diags = check(src);
        let nesting_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id == "readability-deep-nesting")
            .collect();
        assert_eq!(nesting_diags.len(), 3);
    }

    #[test]
    fn for_loop_nesting() {
        let src = concat!(
            "def foo():\n",
            "    for x in y:\n",
            "        for x in y:\n",
            "            for x in y:\n",
            "                for x in y:\n",
            "                    for x in y:\n",
            "                        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn while_loop_nesting() {
        let src = concat!(
            "def foo():\n",
            "    while True:\n",
            "        while True:\n",
            "            while True:\n",
            "                while True:\n",
            "                    while True:\n",
            "                        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn with_statement_nesting() {
        let src = concat!(
            "def foo():\n",
            "    with open('a'):\n",
            "        with open('b'):\n",
            "            with open('c'):\n",
            "                with open('d'):\n",
            "                    with open('e'):\n",
            "                        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn try_except_nesting() {
        let src = concat!(
            "def foo():\n",
            "    try:\n",
            "        try:\n",
            "            try:\n",
            "                try:\n",
            "                    try:\n",
            "                        pass\n",
            "                    except:\n",
            "                        pass\n",
            "                except:\n",
            "                    pass\n",
            "            except:\n",
            "                pass\n",
            "        except:\n",
            "            pass\n",
            "    except:\n",
            "        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn mixed_block_nesting() {
        let src = concat!(
            "def foo():\n",
            "    if True:\n",
            "        for x in y:\n",
            "            while True:\n",
            "                with open('f'):\n",
            "                    try:\n",
            "                        pass\n",
            "                    except:\n",
            "                        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn custom_config_higher_limits() {
        let src = "def foo():\n    pass\n";
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        let config = ComplexityConfig {
            max_function_lines: 50,
            max_class_lines: 300,
            max_file_lines: 1,
            max_nesting_depth: 4,
        };
        let diags = check_complexity(&source, &config);
        assert_eq!(rule_ids(&diags), vec!["readability-file-size"]);
    }

    #[test]
    fn rule_id_has_readability_prefix() {
        let mut src = "def foo():\n".to_string();
        for i in 0..55 {
            src.push_str(&format!("    x = {}\n", i));
        }
        let diags = check(&src);
        assert!(diags[0].rule_id.starts_with("readability-"));
    }

    #[test]
    fn nested_function_length_counted_independently() {
        let mut src = "def outer():\n".to_string();
        src.push_str("    def inner():\n");
        for i in 0..55 {
            src.push_str(&format!("        x = {}\n", i));
        }
        let diags = check(&src);
        let fn_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.rule_id == "readability-function-size")
            .collect();
        assert_eq!(fn_diags.len(), 2);
        assert_eq!(fn_diags[0].line, 1);
        assert_eq!(fn_diags[1].line, 2);
    }

    #[test]
    fn class_inside_function_not_flagged_for_class_size() {
        let mut src = "def foo():\n".to_string();
        src.push_str("    class Bar:\n");
        for i in 0..5 {
            src.push_str(&format!("        x = {}\n", i));
        }
        src.push_str("    pass\n");
        let diags = check(&src);
        assert!(!diags.iter().any(|d| d.rule_id == "readability-class-size"));
    }

    #[test]
    fn parse_error_returns_empty() {
        let src = "def foo(:\n";
        let diags = check(src);
        assert!(diags.is_empty());
    }

    #[test]
    fn else_branch_nesting_same_as_if_body() {
        let src = concat!(
            "def foo():\n",
            "    if True:\n",
            "        if True:\n",
            "            if True:\n",
            "                if True:\n",
            "                    pass\n",
            "                else:\n",
            "                    if True:\n",
            "                        pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn elif_does_not_double_count_depth() {
        let src = concat!(
            "def foo():\n",
            "    if True:\n",
            "        if True:\n",
            "            if True:\n",
            "                pass\n",
            "    elif True:\n",
            "        if True:\n",
            "            if True:\n",
            "                pass\n",
        );
        let diags = check(src);
        assert!(!diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn match_statement_nesting() {
        let src = concat!(
            "def foo():\n",
            "    match x:\n",
            "        case 1:\n",
            "            match x:\n",
            "                case 2:\n",
            "                    match x:\n",
            "                        case 3:\n",
            "                            match x:\n",
            "                                case 4:\n",
            "                                    pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn for_else_branch_nesting() {
        let src = concat!(
            "def foo():\n",
            "    for x in y:\n",
            "        pass\n",
            "    else:\n",
            "        if True:\n",
            "            if True:\n",
            "                if True:\n",
            "                    pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }

    #[test]
    fn while_else_branch_nesting() {
        let src = concat!(
            "def foo():\n",
            "    while True:\n",
            "        pass\n",
            "    else:\n",
            "        if True:\n",
            "            if True:\n",
            "                if True:\n",
            "                    pass\n",
        );
        let diags = check(src);
        assert!(diags.iter().any(|d| d.rule_id == "readability-deep-nesting"));
    }
}
