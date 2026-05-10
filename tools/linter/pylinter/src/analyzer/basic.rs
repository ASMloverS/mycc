use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;
use rustpython_parser::ast::{self, Ranged};

pub fn check(source: &SourceFile, _config: &AnalysisConfig) -> Vec<Diagnostic> {
    let Ok(ast) =
        rustpython_parser::parse(&source.content, rustpython_parser::Mode::Module, "<input>")
    else {
        return vec![];
    };
    let body = match &ast {
        ast::Mod::Module(m) => &m.body,
        _ => return vec![],
    };
    let lines = source.lines();
    let mut locator = rustpython_parser::source_code::RandomLocator::new(&source.content);
    let mut diags = Vec::new();
    walk_mutable_defaults(body, &lines, source, &mut locator, &mut diags);
    walk_missing_self(body, &lines, source, &mut locator, &mut diags);
    walk_bare_except(body, &lines, source, &mut locator, &mut diags);
    walk_none_comparison(body, &lines, source, &mut locator, &mut diags);
    diags
}

// ── Mutable default arguments ──────────────────────────────────────────────

fn walk_mutable_defaults(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                check_defaults(&f.args, lines, source, locator, diags);
                walk_mutable_defaults(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                check_defaults(&f.args, lines, source, locator, diags);
                walk_mutable_defaults(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_mutable_defaults(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                walk_mutable_defaults(&i.body, lines, source, locator, diags);
                walk_mutable_defaults(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                walk_mutable_defaults(&f.body, lines, source, locator, diags);
                walk_mutable_defaults(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_mutable_defaults(&f.body, lines, source, locator, diags);
                walk_mutable_defaults(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_mutable_defaults(&w.body, lines, source, locator, diags);
                walk_mutable_defaults(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_mutable_defaults(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_mutable_defaults(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_mutable_defaults(&t.body, lines, source, locator, diags);
                walk_mutable_defaults(&t.orelse, lines, source, locator, diags);
                walk_mutable_defaults(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_mutable_defaults(&t.body, lines, source, locator, diags);
                walk_mutable_defaults(&t.orelse, lines, source, locator, diags);
                walk_mutable_defaults(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_mutable_defaults(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_defaults(
    args: &ast::Arguments,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for arg in args.posonlyargs.iter().chain(args.args.iter()).chain(args.kwonlyargs.iter()) {
        if let Some(default) = &arg.default {
            if is_mutable_default(default) {
                let line = locator.locate(default.start()).row.to_usize();
                let source_line = lines.get(line - 1).copied().unwrap_or("");
                diags.push(Diagnostic::new_with_source(
                    source.display_path(),
                    line,
                    1,
                    Severity::Warning,
                    "bugprone-mutable-default",
                    "Mutable default argument",
                    source_line,
                ));
            }
        }
    }
}

fn is_mutable_default(expr: &ast::Expr) -> bool {
    matches!(
        expr,
        ast::Expr::List(_) | ast::Expr::Dict(_) | ast::Expr::Set(_)
    )
}

// ── Missing self ────────────────────────────────────────────────────────────

fn walk_missing_self(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::ClassDef(c) => {
                for member in &c.body {
                    let (func_args, func_name, func_start) = match member {
                        ast::Stmt::FunctionDef(f) => (&f.args, f.name.as_str(), f.start()),
                        ast::Stmt::AsyncFunctionDef(f) => (&f.args, f.name.as_str(), f.start()),
                        _ => continue,
                    };
                    if matches!(
                        func_name,
                        "__init_subclass__" | "__class_getitem__"
                    ) {
                        continue;
                    }
                    if has_decorator(member, "staticmethod") {
                        continue;
                    }
                    if func_args.posonlyargs.is_empty() && func_args.args.is_empty() {
                        let line = locator.locate(func_start).row.to_usize();
                        let source_line = lines.get(line - 1).copied().unwrap_or("");
                        diags.push(Diagnostic::new_with_source(
                            source.display_path(),
                            line,
                            1,
                            Severity::Error,
                            "bugprone-missing-self",
                            "Method has no parameters; expected 'self' or 'cls'",
                            source_line,
                        ));
                        continue;
                    }
                    let first_name = if !func_args.posonlyargs.is_empty() {
                        func_args.posonlyargs[0].def.arg.as_str()
                    } else {
                        func_args.args[0].def.arg.as_str()
                    };
                    if first_name != "self" && first_name != "cls" {
                        let line = locator.locate(func_start).row.to_usize();
                        let source_line = lines.get(line - 1).copied().unwrap_or("");
                        diags.push(Diagnostic::new_with_source(
                            source.display_path(),
                            line,
                            1,
                            Severity::Error,
                            "bugprone-missing-self",
                            &format!(
                                "Method first parameter is '{}' ; expected 'self' or 'cls'",
                                first_name
                            ),
                            source_line,
                        ));
                    }
                }
            }
            ast::Stmt::If(i) => {
                walk_missing_self(&i.body, lines, source, locator, diags);
                walk_missing_self(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                walk_missing_self(&f.body, lines, source, locator, diags);
                walk_missing_self(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_missing_self(&f.body, lines, source, locator, diags);
                walk_missing_self(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_missing_self(&w.body, lines, source, locator, diags);
                walk_missing_self(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_missing_self(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_missing_self(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_missing_self(&t.body, lines, source, locator, diags);
                walk_missing_self(&t.orelse, lines, source, locator, diags);
                walk_missing_self(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_missing_self(&t.body, lines, source, locator, diags);
                walk_missing_self(&t.orelse, lines, source, locator, diags);
                walk_missing_self(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_missing_self(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn has_decorator(stmt: &ast::Stmt, name: &str) -> bool {
    let decorators = match stmt {
        ast::Stmt::FunctionDef(f) => &f.decorator_list,
        ast::Stmt::AsyncFunctionDef(f) => &f.decorator_list,
        _ => return false,
    };
    decorators.iter().any(|d| {
        if let ast::Expr::Name(n) = d {
            n.id.as_str() == name
        } else {
            false
        }
    })
}

// ── Bare except ─────────────────────────────────────────────────────────────

fn walk_bare_except(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::Try(t) => {
                check_handlers(&t.handlers, lines, source, locator, diags);
                walk_bare_except(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_bare_except(&h.body, lines, source, locator, diags);
                }
                walk_bare_except(&t.orelse, lines, source, locator, diags);
                walk_bare_except(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                check_handlers(&t.handlers, lines, source, locator, diags);
                walk_bare_except(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_bare_except(&h.body, lines, source, locator, diags);
                }
                walk_bare_except(&t.orelse, lines, source, locator, diags);
                walk_bare_except(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::FunctionDef(f) => {
                walk_bare_except(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                walk_bare_except(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_bare_except(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                walk_bare_except(&i.body, lines, source, locator, diags);
                walk_bare_except(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                walk_bare_except(&f.body, lines, source, locator, diags);
                walk_bare_except(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_bare_except(&f.body, lines, source, locator, diags);
                walk_bare_except(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_bare_except(&w.body, lines, source, locator, diags);
                walk_bare_except(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_bare_except(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_bare_except(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_bare_except(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_handlers(
    handlers: &[ast::ExceptHandler],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        if h.type_.is_none() {
            let line = locator.locate(handler.start()).row.to_usize();
            let source_line = lines.get(line - 1).copied().unwrap_or("");
            diags.push(Diagnostic::new_with_source(
                source.display_path(),
                line,
                1,
                Severity::Warning,
                "bugprone-bare-except",
                "Bare except clause catches all exceptions including SystemExit and KeyboardInterrupt",
                source_line,
            ));
        }
    }
}

// ── == None comparison ──────────────────────────────────────────────────────

fn walk_none_comparison(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                walk_none_comparison(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                walk_none_comparison(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_none_comparison(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                check_expr_for_none_cmp(&i.test, lines, source, locator, diags);
                walk_none_comparison(&i.body, lines, source, locator, diags);
                walk_none_comparison(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                check_expr_for_none_cmp(&w.test, lines, source, locator, diags);
                walk_none_comparison(&w.body, lines, source, locator, diags);
                walk_none_comparison(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                walk_none_comparison(&f.body, lines, source, locator, diags);
                walk_none_comparison(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_none_comparison(&f.body, lines, source, locator, diags);
                walk_none_comparison(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_none_comparison(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_none_comparison(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Return(r) => {
                if let Some(expr) = &r.value {
                    check_expr_for_none_cmp(expr, lines, source, locator, diags);
                }
            }
            ast::Stmt::Assign(a) => {
                check_expr_for_none_cmp(&a.value, lines, source, locator, diags);
            }
            ast::Stmt::AnnAssign(a) => {
                if let Some(v) = &a.value {
                    check_expr_for_none_cmp(v, lines, source, locator, diags);
                }
            }
            ast::Stmt::Assert(a) => {
                check_expr_for_none_cmp(&a.test, lines, source, locator, diags);
            }
            ast::Stmt::Expr(e) => {
                check_expr_for_none_cmp(&e.value, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_none_comparison(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_none_comparison(&h.body, lines, source, locator, diags);
                }
                walk_none_comparison(&t.orelse, lines, source, locator, diags);
                walk_none_comparison(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_none_comparison(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_none_comparison(&h.body, lines, source, locator, diags);
                }
                walk_none_comparison(&t.orelse, lines, source, locator, diags);
                walk_none_comparison(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_none_comparison(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn report_none_cmp(
    is_eq: bool,
    cmp: &ast::ExprCompare,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    let op_str = if is_eq { "==" } else { "!=" };
    let suggestion = if is_eq { "is None" } else { "is not None" };
    let line = locator.locate(cmp.start()).row.to_usize();
    let source_line = lines.get(line - 1).copied().unwrap_or("");
    diags.push(Diagnostic::new_with_source(
        source.display_path(),
        line,
        1,
        Severity::Warning,
        "bugprone-none-comparison",
        &format!(
            "Comparison to None using '{}'; use '{}' instead",
            op_str, suggestion
        ),
        source_line,
    ));
}

fn check_expr_for_none_cmp(
    expr: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    if let ast::Expr::Compare(cmp) = expr {
        if !cmp.ops.is_empty()
            && is_none_constant(&cmp.left)
            && matches!(cmp.ops[0], ast::CmpOp::Eq | ast::CmpOp::NotEq)
        {
            report_none_cmp(
                matches!(cmp.ops[0], ast::CmpOp::Eq),
                cmp, lines, source, locator, diags,
            );
        }
        for (i, op) in cmp.ops.iter().enumerate() {
            if !matches!(op, ast::CmpOp::Eq | ast::CmpOp::NotEq) {
                continue;
            }
            if let Some(comparator) = cmp.comparators.get(i) {
                if is_none_constant(comparator) {
                    report_none_cmp(
                        matches!(op, ast::CmpOp::Eq),
                        cmp, lines, source, locator, diags,
                    );
                }
            }
        }
    }
    walk_expr_for_none_cmp(expr, lines, source, locator, diags);
}

fn walk_expr_for_none_cmp(
    expr: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::Expr::BoolOp(b) => {
            for v in &b.values {
                check_expr_for_none_cmp(v, lines, source, locator, diags);
            }
        }
        ast::Expr::BinOp(b) => {
            check_expr_for_none_cmp(&b.left, lines, source, locator, diags);
            check_expr_for_none_cmp(&b.right, lines, source, locator, diags);
        }
        ast::Expr::UnaryOp(u) => {
            check_expr_for_none_cmp(&u.operand, lines, source, locator, diags);
        }
        ast::Expr::IfExp(i) => {
            check_expr_for_none_cmp(&i.test, lines, source, locator, diags);
            check_expr_for_none_cmp(&i.body, lines, source, locator, diags);
            check_expr_for_none_cmp(&i.orelse, lines, source, locator, diags);
        }
        ast::Expr::Call(c) => {
            for arg in &c.args {
                check_expr_for_none_cmp(arg, lines, source, locator, diags);
            }
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                check_expr_for_none_cmp(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                check_expr_for_none_cmp(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::Set(s) => {
            for elt in &s.elts {
                check_expr_for_none_cmp(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::Dict(d) => {
            for k in d.keys.iter().flatten() {
                check_expr_for_none_cmp(k, lines, source, locator, diags);
            }
            for v in &d.values {
                check_expr_for_none_cmp(v, lines, source, locator, diags);
            }
        }
        ast::Expr::Compare(cmp) => {
            check_expr_for_none_cmp(&cmp.left, lines, source, locator, diags);
            for c in &cmp.comparators {
                check_expr_for_none_cmp(c, lines, source, locator, diags);
            }
        }
        ast::Expr::Subscript(s) => {
            check_expr_for_none_cmp(&s.value, lines, source, locator, diags);
        }
        ast::Expr::Starred(s) => {
            check_expr_for_none_cmp(&s.value, lines, source, locator, diags);
        }
        ast::Expr::Attribute(a) => {
            check_expr_for_none_cmp(&a.value, lines, source, locator, diags);
        }
        _ => {}
    }
}

fn is_none_constant(expr: &ast::Expr) -> bool {
    matches!(expr, ast::Expr::Constant(c) if matches!(c.value, ast::Constant::None))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn analyze(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check(&source, &AnalysisConfig::default())
    }

    fn rule_ids(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule_id.as_str()).collect()
    }

    // ── Mutable default ──────────────────────────────────────────────────

    #[test]
    fn mutable_default_list() {
        let diags = analyze("def foo(x=[]):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-mutable-default"]);
    }

    #[test]
    fn mutable_default_dict() {
        let diags = analyze("def foo(x={}):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-mutable-default"]);
    }

    #[test]
    fn mutable_default_set() {
        let diags = analyze("def foo(x=set()):\n    pass\n");
        // set() is a Call, not a Set literal — not flagged by this rule
        assert!(!rule_ids(&diags).contains(&"bugprone-mutable-default"));
    }

    #[test]
    fn mutable_default_set_literal() {
        let diags = analyze("def foo(x={1}):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-mutable-default"]);
    }

    #[test]
    fn immutable_default_ok() {
        let diags = analyze("def foo(x=None):\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-mutable-default"));
    }

    #[test]
    fn immutable_default_int_ok() {
        let diags = analyze("def foo(x=0):\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-mutable-default"));
    }

    #[test]
    fn immutable_default_tuple_ok() {
        let diags = analyze("def foo(x=()):\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-mutable-default"));
    }

    #[test]
    fn mutable_default_kw_only() {
        let diags = analyze("def foo(*, x=[]):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-mutable-default"]);
    }

    #[test]
    fn mutable_default_nested_function() {
        let diags = analyze("def outer():\n    def inner(x=[]):\n        pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-mutable-default"]);
    }

    // ── Missing self ─────────────────────────────────────────────────────

    #[test]
    fn missing_self() {
        let diags = analyze("class Foo:\n    def bar():\n        pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-missing-self"]);
    }

    #[test]
    fn missing_self_no_params() {
        let diags = analyze("class Foo:\n    def bar():\n        pass\n");
        assert!(rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn has_self_ok() {
        let diags = analyze("class Foo:\n    def bar(self):\n        pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn has_cls_ok() {
        let diags = analyze("class Foo:\n    def bar(cls):\n        pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn static_method_ok() {
        let diags = analyze("class Foo:\n    @staticmethod\n    def bar():\n        pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn init_subclass_ok() {
        let diags = analyze("class Foo:\n    def __init_subclass__(cls):\n        pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn class_getitem_ok() {
        let diags = analyze("class Foo:\n    def __class_getitem__(cls, item):\n        pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn wrong_first_param() {
        let diags = analyze("class Foo:\n    def bar(other):\n        pass\n");
        assert!(rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    #[test]
    fn function_not_in_class_ok() {
        let diags = analyze("def bar():\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-missing-self"));
    }

    // ── Bare except ──────────────────────────────────────────────────────

    #[test]
    fn bare_except() {
        let diags = analyze("try:\n    pass\nexcept:\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-bare-except"]);
    }

    #[test]
    fn specific_except_ok() {
        let diags = analyze("try:\n    pass\nexcept ValueError:\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-bare-except"));
    }

    #[test]
    fn except_with_name_ok() {
        let diags = analyze("try:\n    pass\nexcept Exception as e:\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-bare-except"));
    }

    #[test]
    fn bare_except_in_function() {
        let diags = analyze("def foo():\n    try:\n        pass\n    except:\n        pass\n");
        assert!(rule_ids(&diags).contains(&"bugprone-bare-except"));
    }

    // ── == None comparison ───────────────────────────────────────────────

    #[test]
    fn none_comparison_eq() {
        let diags = analyze("if x == None:\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-none-comparison"]);
    }

    #[test]
    fn none_comparison_neq() {
        let diags = analyze("if x != None:\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["bugprone-none-comparison"]);
    }

    #[test]
    fn none_is_ok() {
        let diags = analyze("if x is None:\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-none-comparison"));
    }

    #[test]
    fn none_is_not_ok() {
        let diags = analyze("if x is not None:\n    pass\n");
        assert!(!rule_ids(&diags).contains(&"bugprone-none-comparison"));
    }

    #[test]
    fn none_comparison_in_assignment() {
        let diags = analyze("x = (y == None)\n");
        assert!(rule_ids(&diags).contains(&"bugprone-none-comparison"));
    }

    #[test]
    fn none_comparison_in_while() {
        let diags = analyze("while x == None:\n    pass\n");
        assert!(rule_ids(&diags).contains(&"bugprone-none-comparison"));
    }

    #[test]
    fn none_comparison_message_eq() {
        let diags = analyze("if x == None:\n    pass\n");
        let d = diags.iter().find(|d| d.rule_id == "bugprone-none-comparison").unwrap();
        assert!(d.message.contains("'is None'"));
    }

    #[test]
    fn none_comparison_message_neq() {
        let diags = analyze("if x != None:\n    pass\n");
        let d = diags.iter().find(|d| d.rule_id == "bugprone-none-comparison").unwrap();
        assert!(d.message.contains("'is not None'"));
    }

    // ── Integration ──────────────────────────────────────────────────────

    #[test]
    fn empty_source_no_diags() {
        let diags = analyze("");
        assert!(diags.is_empty());
    }

    #[test]
    fn clean_code_no_diags() {
        let src = "class Foo:\n    def bar(self):\n        if x is None:\n            pass\n";
        let diags = analyze(src);
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_issues() {
        let src = "class Foo:\n    def bar():\n        pass\n\ndef baz(x=[]):\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-missing-self"));
        assert!(rule_ids(&diags).contains(&"bugprone-mutable-default"));
    }

    // ── Code review fix verification ─────────────────────────────────────

    #[test]
    fn bare_except_nested_in_handler() {
        let src = "try:\n    pass\nexcept ValueError:\n    try:\n        pass\n    except:\n        pass\n";
        assert!(rule_ids(&analyze(src)).contains(&"bugprone-bare-except"));
    }

    #[test]
    fn none_comparison_lhs_eq() {
        let diags = analyze("if None == x:\n    pass\n");
        assert!(rule_ids(&diags).contains(&"bugprone-none-comparison"));
    }

    #[test]
    fn none_comparison_lhs_neq() {
        let diags = analyze("if None != x:\n    pass\n");
        assert!(rule_ids(&diags).contains(&"bugprone-none-comparison"));
    }

    #[test]
    fn bare_except_in_async_for() {
        let src = "async def foo():\n    async for x in y:\n        try:\n            pass\n        except:\n            pass\n";
        assert!(rule_ids(&analyze(src)).contains(&"bugprone-bare-except"));
    }

    #[test]
    fn mutable_default_in_async_for() {
        let src = "async def foo():\n    async for x in y:\n        def inner(z=[]):\n            pass\n";
        assert!(rule_ids(&analyze(src)).contains(&"bugprone-mutable-default"));
    }

    #[test]
    fn missing_self_in_nested_class_in_if() {
        let src = "if True:\n    class Foo:\n        def bar():\n            pass\n";
        assert!(rule_ids(&analyze(src)).contains(&"bugprone-missing-self"));
    }
}
