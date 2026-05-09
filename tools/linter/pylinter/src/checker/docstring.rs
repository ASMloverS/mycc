use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::DocstringConfig;
use rustpython_parser::ast::{self, Ranged};

pub fn check_docstring(source: &SourceFile, config: &DocstringConfig) -> Vec<Diagnostic> {
    if !config.module && !config.class && !config.function {
        return vec![];
    }

    let mut diags = Vec::new();

    let Ok(ast) =
        rustpython_parser::parse(&source.content, rustpython_parser::Mode::Module, "<input>")
    else {
        return diags;
    };

    let lines = source.lines();
    let locator = rustpython_parser::source_code::RandomLocator::new(&source.content);

    let body = match &ast {
        ast::Mod::Module(m) => &m.body,
        _ => return diags,
    };

    let mut ctx = WalkContext {
        lines: &lines,
        source,
        locator,
        config,
        diags: &mut diags,
    };

    if config.module && !has_docstring(body) {
        ctx.emit_at(1, "readability-missing-module-docstring", "Missing module docstring");
    }

    walk_stmts(body, &mut ctx);

    diags
}

fn has_docstring(body: &[ast::Stmt]) -> bool {
    let Some(first) = body.first() else {
        return false;
    };
    let ast::Stmt::Expr(expr_stmt) = first else {
        return false;
    };
    matches!(
        expr_stmt.value.as_ref(),
        ast::Expr::Constant(c) if matches!(&c.value, ast::Constant::Str(_))
    )
}

struct WalkContext<'a> {
    lines: &'a [&'a str],
    source: &'a SourceFile,
    locator: rustpython_parser::source_code::RandomLocator<'a>,
    config: &'a DocstringConfig,
    diags: &'a mut Vec<Diagnostic>,
}

impl<'a> WalkContext<'a> {
    fn emit(&mut self, pos: ast::TextSize, rule_id: &str, message: &str) {
        let line = self.locator.locate(pos).row.to_usize();
        self.push_diag(line, rule_id, message);
    }

    fn emit_at(&mut self, line: usize, rule_id: &str, message: &str) {
        self.push_diag(line, rule_id, message);
    }

    fn push_diag(&mut self, line: usize, rule_id: &str, message: &str) {
        let source_line = self.lines.get(line - 1).copied().unwrap_or("");
        self.diags.push(Diagnostic::new_with_source(
            self.source.display_path(),
            line,
            1,
            Severity::Warning,
            rule_id,
            message,
            source_line,
        ));
    }
}

fn walk_handlers(handlers: &[ast::ExceptHandler], ctx: &mut WalkContext) {
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        walk_stmts(&h.body, ctx);
    }
}

fn check_function_def(body: &[ast::Stmt], pos: ast::TextSize, ctx: &mut WalkContext) {
    if ctx.config.function && !has_docstring(body) {
        ctx.emit(pos, "readability-missing-function-docstring", "Missing function docstring");
    }
    walk_stmts(body, ctx);
}

fn walk_try(
    body: &[ast::Stmt],
    handlers: &[ast::ExceptHandler],
    orelse: &[ast::Stmt],
    finalbody: &[ast::Stmt],
    ctx: &mut WalkContext,
) {
    walk_stmts(body, ctx);
    walk_handlers(handlers, ctx);
    walk_stmts(orelse, ctx);
    walk_stmts(finalbody, ctx);
}

fn walk_stmts(stmts: &[ast::Stmt], ctx: &mut WalkContext) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::ClassDef(c) => {
                if ctx.config.class && !has_docstring(&c.body) {
                    ctx.emit(c.start(), "readability-missing-class-docstring", "Missing class docstring");
                }
                walk_stmts(&c.body, ctx);
            }
            ast::Stmt::FunctionDef(f) => check_function_def(&f.body, f.start(), ctx),
            ast::Stmt::AsyncFunctionDef(f) => check_function_def(&f.body, f.start(), ctx),
            ast::Stmt::If(i) => {
                walk_stmts(&i.body, ctx);
                walk_stmts(&i.orelse, ctx);
            }
            ast::Stmt::For(f) => {
                walk_stmts(&f.body, ctx);
                walk_stmts(&f.orelse, ctx);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_stmts(&f.body, ctx);
                walk_stmts(&f.orelse, ctx);
            }
            ast::Stmt::While(w) => {
                walk_stmts(&w.body, ctx);
                walk_stmts(&w.orelse, ctx);
            }
            ast::Stmt::With(w) => {
                walk_stmts(&w.body, ctx);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_stmts(&w.body, ctx);
            }
            ast::Stmt::Try(t) => {
                walk_try(&t.body, &t.handlers, &t.orelse, &t.finalbody, ctx);
            }
            ast::Stmt::TryStar(t) => {
                walk_try(&t.body, &t.handlers, &t.orelse, &t.finalbody, ctx);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_stmts(&case.body, ctx);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn check(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_docstring(&source, &DocstringConfig::default())
    }

    fn check_with_config(src: &str, config: &DocstringConfig) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_docstring(&source, config)
    }

    fn rule_ids(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule_id.as_str()).collect()
    }

    #[test]
    fn module_with_docstring_no_warning() {
        let diags = check("\"\"\"Module doc.\"\"\"\nx = 1\n");
        assert!(diags.iter().all(|d| !d.rule_id.contains("module")));
    }

    #[test]
    fn module_without_docstring_warning() {
        let diags = check("x = 1\n");
        assert_eq!(
            rule_ids(&diags),
            vec!["readability-missing-module-docstring"]
        );
    }

    #[test]
    fn class_with_docstring_no_warning() {
        let diags = check("class Foo:\n    \"\"\"Class doc.\"\"\"\n    pass\n");
        assert!(diags.iter().all(|d| !d.rule_id.contains("class")));
    }

    #[test]
    fn class_without_docstring_warning() {
        let diags = check("class Foo:\n    pass\n");
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-class-docstring"));
    }

    #[test]
    fn function_with_docstring_no_warning() {
        let diags = check("def foo():\n    \"\"\"Doc.\"\"\"\n    pass\n");
        assert!(diags.iter().all(|d| !d.rule_id.contains("function")));
    }

    #[test]
    fn function_without_docstring_warning() {
        let diags = check("def foo():\n    pass\n");
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-function-docstring"));
    }

    #[test]
    fn all_checks_disabled_no_warnings() {
        let config = DocstringConfig {
            module: false,
            class: false,
            function: false,
        };
        let diags = check_with_config("x = 1\nclass Foo:\n    pass\ndef bar():\n    pass\n", &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn async_function_without_docstring_warning() {
        let diags = check("async def foo():\n    pass\n");
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-function-docstring"));
    }

    #[test]
    fn async_function_with_docstring_no_warning() {
        let diags = check("async def foo():\n    \"\"\"Doc.\"\"\"\n    pass\n");
        assert!(diags.iter().all(|d| !d.rule_id.contains("function")));
    }

    #[test]
    fn nested_class_handling() {
        let diags = check(
            "class Outer:\n    \"\"\"Outer doc.\"\"\"\n    class Inner:\n        pass\n",
        );
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-class-docstring" && d.line == 3));
    }

    #[test]
    fn nested_function_handling() {
        let diags = check(
            "def outer():\n    \"\"\"Outer doc.\"\"\"\n    def inner():\n        pass\n",
        );
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-function-docstring" && d.line == 3));
    }

    #[test]
    fn module_only_class_disabled() {
        let config = DocstringConfig {
            module: true,
            class: false,
            function: false,
        };
        let diags = check_with_config("class Foo:\n    pass\n", &config);
        assert!(diags.iter().all(|d| !d.rule_id.contains("class")));
        assert!(diags.iter().any(|d| d.rule_id.contains("module")));
    }

    #[test]
    fn function_only_enabled() {
        let config = DocstringConfig {
            module: false,
            class: false,
            function: true,
        };
        let diags = check_with_config("def foo():\n    pass\n", &config);
        assert_eq!(
            rule_ids(&diags),
            vec!["readability-missing-function-docstring"]
        );
    }

    #[test]
    fn parse_error_returns_empty() {
        let diags = check("def foo(:\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn diagnostic_contains_source_line() {
        let diags = check("class Foo:\n    pass\n");
        let class_diag = diags.iter().find(|d| d.rule_id.contains("class")).unwrap();
        assert_eq!(class_diag.source_line.as_deref(), Some("class Foo:"));
    }

    #[test]
    fn rule_id_has_readability_prefix() {
        let diags = check("x = 1\n");
        assert!(diags[0].rule_id.starts_with("readability-"));
    }

    #[test]
    fn all_three_missing() {
        let diags = check("class Foo:\n    def bar(self):\n        pass\n");
        let ids = rule_ids(&diags);
        assert!(ids.contains(&"readability-missing-module-docstring"));
        assert!(ids.contains(&"readability-missing-class-docstring"));
        assert!(ids.contains(&"readability-missing-function-docstring"));
    }

    #[test]
    fn empty_file_module_docstring_warning() {
        let diags = check("");
        // Empty file: module body is empty → missing module docstring
        assert_eq!(
            rule_ids(&diags),
            vec!["readability-missing-module-docstring"]
        );
    }

    #[test]
    fn single_line_string_docstring() {
        let diags = check("\"Single line docstring.\"\nx = 1\n");
        assert!(diags.iter().all(|d| !d.rule_id.contains("module")));
    }

    #[test]
    fn method_inside_class() {
        let diags = check("class Foo:\n    \"\"\"Foo doc.\"\"\"\n    def bar(self):\n        pass\n");
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-function-docstring"));
    }

    #[test]
    fn function_in_except_without_docstring() {
        let diags = check(
            "try:\n    pass\nexcept:\n    def foo():\n        pass\n",
        );
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-function-docstring"));
    }

    #[test]
    fn function_in_except_with_docstring_no_warning() {
        let diags = check(
            "try:\n    pass\nexcept:\n    def foo():\n        \"\"\"Doc.\"\"\"\n        pass\n",
        );
        assert!(diags.iter().all(|d| !d.rule_id.contains("function")));
    }

    #[test]
    fn class_in_try_star_except_without_docstring() {
        let diags = check(
            "try:\n    pass\nexcept* ValueError:\n    class Foo:\n        pass\n",
        );
        assert!(diags.iter().any(|d| d.rule_id == "readability-missing-class-docstring"));
    }
}
