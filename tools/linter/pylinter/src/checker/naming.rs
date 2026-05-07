use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::{NamingConfig, NamingStyle};
use regex::Regex;
use rustpython_parser::ast::{self, Ranged};
use std::sync::LazyLock;

static SNAKE_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z_][a-z0-9_]*$").unwrap());
static UPPER_SNAKE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z_][A-Z0-9_]*$").unwrap());
static PASCAL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Z][a-zA-Z0-9]*$").unwrap());
static CAMEL_CASE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-zA-Z0-9]*$").unwrap());

fn naming_regex(style: &NamingStyle) -> &'static Regex {
    match style {
        NamingStyle::SnakeCase => &SNAKE_CASE_RE,
        NamingStyle::UpperSnakeCase => &UPPER_SNAKE_RE,
        NamingStyle::PascalCase => &PASCAL_CASE_RE,
        NamingStyle::CamelCase => &CAMEL_CASE_RE,
    }
}

fn is_upper_snake(name: &str) -> bool {
    UPPER_SNAKE_RE.is_match(name)
}

pub fn check_naming(source: &SourceFile, config: &NamingConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Ok(ast) =
        rustpython_parser::parse(&source.content, rustpython_parser::Mode::Module, "<input>")
    else {
        return diags;
    };

    let lines = source.lines();
    let mut locator =
        rustpython_parser::source_code::RandomLocator::new(&source.content);

    check_module_name(source, config, &mut diags, &lines);
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
    );

    diags
}

fn check_module_name(
    source: &SourceFile,
    config: &NamingConfig,
    diags: &mut Vec<Diagnostic>,
    lines: &[&str],
) {
    let file_name = source
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if file_name == "__init__" || file_name.is_empty() {
        return;
    }
    check_name(file_name, &config.module, "module", 1, source, diags, lines);
}

fn check_assign_name(
    name: &str,
    config: &NamingConfig,
    line: usize,
    source: &SourceFile,
    diags: &mut Vec<Diagnostic>,
    lines: &[&str],
) {
    let (style, kind) = if is_upper_snake(name) {
        (&config.constant, "constant")
    } else {
        (&config.variable, "variable")
    };
    check_name(name, style, kind, line, source, diags, lines);
}

fn walk_with_items(
    items: &[ast::WithItem],
    lines: &[&str],
    source: &SourceFile,
    config: &NamingConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for item in items {
        if let Some(vars) = &item.optional_vars {
            walk_expr(vars, lines, source, config, locator, diags);
        }
    }
}

fn walk_handlers(
    handlers: &[ast::ExceptHandler],
    lines: &[&str],
    source: &SourceFile,
    config: &NamingConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        walk_stmts(&h.body, lines, source, config, locator, diags);
    }
}

fn walk_stmts(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    config: &NamingConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                let line = locator.locate(f.start()).row.to_usize();
                check_name(f.name.as_str(), &config.function, "function", line, source, diags, lines);
                walk_stmts(&f.body, lines, source, config, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                let line = locator.locate(f.start()).row.to_usize();
                check_name(f.name.as_str(), &config.function, "function", line, source, diags, lines);
                walk_stmts(&f.body, lines, source, config, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                let line = locator.locate(c.start()).row.to_usize();
                check_name(c.name.as_str(), &config.class, "class", line, source, diags, lines);
                walk_stmts(&c.body, lines, source, config, locator, diags);
            }
            ast::Stmt::Assign(a) => {
                for target in &a.targets {
                    if let ast::Expr::Name(name_node) = target {
                        let line = locator.locate(name_node.start()).row.to_usize();
                        check_assign_name(name_node.id.as_str(), config, line, source, diags, lines);
                    }
                }
            }
            ast::Stmt::AnnAssign(a) => {
                if let ast::Expr::Name(name_node) = a.target.as_ref() {
                    let line = locator.locate(name_node.start()).row.to_usize();
                    check_assign_name(name_node.id.as_str(), config, line, source, diags, lines);
                }
            }
            ast::Stmt::For(f) => {
                walk_expr(&f.target, lines, source, config, locator, diags);
                walk_stmts(&f.body, lines, source, config, locator, diags);
                walk_stmts(&f.orelse, lines, source, config, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_expr(&f.target, lines, source, config, locator, diags);
                walk_stmts(&f.body, lines, source, config, locator, diags);
                walk_stmts(&f.orelse, lines, source, config, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_stmts(&w.body, lines, source, config, locator, diags);
                walk_stmts(&w.orelse, lines, source, config, locator, diags);
            }
            ast::Stmt::If(i) => {
                walk_stmts(&i.body, lines, source, config, locator, diags);
                walk_stmts(&i.orelse, lines, source, config, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_with_items(&w.items, lines, source, config, locator, diags);
                walk_stmts(&w.body, lines, source, config, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_with_items(&w.items, lines, source, config, locator, diags);
                walk_stmts(&w.body, lines, source, config, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_stmts(&t.body, lines, source, config, locator, diags);
                walk_handlers(&t.handlers, lines, source, config, locator, diags);
                walk_stmts(&t.orelse, lines, source, config, locator, diags);
                walk_stmts(&t.finalbody, lines, source, config, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_stmts(&t.body, lines, source, config, locator, diags);
                walk_handlers(&t.handlers, lines, source, config, locator, diags);
                walk_stmts(&t.orelse, lines, source, config, locator, diags);
                walk_stmts(&t.finalbody, lines, source, config, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_stmts(&case.body, lines, source, config, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn walk_expr(
    expr: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    config: &NamingConfig,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::Expr::Name(name_node) => {
            let line = locator.locate(name_node.start()).row.to_usize();
            check_assign_name(name_node.id.as_str(), config, line, source, diags, lines);
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                walk_expr(elt, lines, source, config, locator, diags);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                walk_expr(elt, lines, source, config, locator, diags);
            }
        }
        ast::Expr::Starred(s) => {
            walk_expr(&s.value, lines, source, config, locator, diags);
        }
        _ => {}
    }
}

fn check_name(
    name: &str,
    style: &NamingStyle,
    kind: &str,
    line_num: usize,
    source: &SourceFile,
    diags: &mut Vec<Diagnostic>,
    lines: &[&str],
) {
    if name.starts_with('_') {
        return;
    }
    let re = naming_regex(style);
    if !re.is_match(name) {
        let source_line = lines.get(line_num - 1).copied().unwrap_or("");
        diags.push(Diagnostic::new_with_source(
            source.display_path(),
            line_num,
            1,
            Severity::Warning,
            &format!("readability-naming-{}", kind),
            &format!(
                "{} '{}' does not follow {} convention",
                kind,
                name,
                style.as_str()
            ),
            source_line,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NamingConfig;
    use crate::common::source::SourceFile;
    use std::path::PathBuf;

    fn check(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_naming(&source, &NamingConfig::default())
    }

    fn rule_ids(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule_id.as_str()).collect()
    }

    #[test]
    fn function_snake_case_violation() {
        let diags = check("def MyFunction():\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-function"]);
    }

    #[test]
    fn function_snake_case_ok() {
        let diags = check("def my_function():\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn async_function_snake_case_violation() {
        let diags = check("async def MyFunction():\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-function"]);
    }

    #[test]
    fn class_pascal_case_violation() {
        let diags = check("class my_class:\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-class"]);
    }

    #[test]
    fn class_pascal_case_ok() {
        let diags = check("class MyClass:\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn constant_upper_snake_violation() {
        let diags = check("myConstant = 42\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-variable"]);
    }

    #[test]
    fn constant_upper_snake_ok() {
        let diags = check("MY_CONSTANT = 42\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn variable_snake_case_violation() {
        let diags = check("def foo():\n    myVar = 1\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-variable"]);
    }

    #[test]
    fn variable_snake_case_ok() {
        let diags = check("def foo():\n    my_var = 1\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn no_warning_correct_naming() {
        let diags = check("def my_function():\n    my_var = 1\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn underscore_prefix_not_flagged() {
        let diags = check("def _private():\n    _MyVar = 1\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn method_inside_class_checked() {
        let diags = check("class MyClass:\n    def BadMethod(self):\n        pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-function"]);
    }

    #[test]
    fn module_name_violation() {
        let source = SourceFile::from_string("x = 1\n", PathBuf::from("MyModule.py"));
        let diags = check_naming(&source, &NamingConfig::default());
        assert_eq!(rule_ids(&diags), vec!["readability-naming-module"]);
    }

    #[test]
    fn module_name_init_skipped() {
        let source = SourceFile::from_string("x = 1\n", PathBuf::from("__init__.py"));
        let diags = check_naming(&source, &NamingConfig::default());
        assert!(diags.is_empty());
    }

    #[test]
    fn for_loop_target_variable() {
        let diags = check("for MyVar in range(10):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-variable"]);
    }

    #[test]
    fn nested_function_checked() {
        let diags = check("def outer():\n    def Inner():\n        pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-function"]);
    }

    #[test]
    fn mixed_violations() {
        let diags = check("class my_class:\n    def BadMethod(self):\n        pass\n");
        assert_eq!(
            rule_ids(&diags),
            vec!["readability-naming-class", "readability-naming-function"]
        );
    }

    #[test]
    fn custom_style_pascal_function() {
        let mut config = NamingConfig::default();
        config.function = NamingStyle::PascalCase;
        let source = SourceFile::from_string("def my_function():\n    pass\n", PathBuf::from("test.py"));
        let diags = check_naming(&source, &config);
        assert_eq!(rule_ids(&diags), vec!["readability-naming-function"]);
    }

    #[test]
    fn annotated_assign_violation() {
        let diags = check("myVar: int = 5\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-variable"]);
    }

    #[test]
    fn annotated_assign_snake_ok() {
        let diags = check("my_var: int = 5\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn with_statement_binding_violation() {
        let diags = check("with open('f') as MyFile:\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-variable"]);
    }

    #[test]
    fn tuple_unpacking_in_for_loop() {
        let diags = check("for x, MyVar in items:\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-naming-variable"]);
    }

    #[test]
    fn dunder_init_not_flagged() {
        let diags = check("class Foo:\n    def __init__(self):\n        pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn dunder_str_not_flagged() {
        let diags = check("class Foo:\n    def __str__(self):\n        return ''\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn diagnostic_contains_source_line() {
        let diags = check("def MyFunction():\n    pass\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].source_line.as_deref(), Some("def MyFunction():"));
    }

    #[test]
    fn rule_id_has_readability_prefix() {
        let diags = check("def MyFunction():\n    pass\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].rule_id.starts_with("readability-naming-"));
    }
}
