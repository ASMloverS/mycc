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
    walk_unnecessary_pass(body, &lines, source, &mut locator, &mut diags);
    walk_empty_fstring(body, &lines, source, &mut locator, &mut diags);
    walk_simplify_if_return(body, &lines, source, &mut locator, &mut diags);
    diags
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn report(
    node_start: rustpython_parser::source_code::SourceLocation,
    rule_id: &str,
    message: &str,
    lines: &[&str],
    source: &SourceFile,
    diags: &mut Vec<Diagnostic>,
) {
    let line = node_start.row.to_usize();
    let source_line = lines.get(line - 1).copied().unwrap_or("");
    diags.push(Diagnostic::new_with_source(
        source.display_path(),
        line,
        1,
        Severity::Warning,
        rule_id,
        message,
        source_line,
    ));
}

// ── Unnecessary pass ────────────────────────────────────────────────────────

fn walk_unnecessary_pass(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                check_body_for_unnecessary_pass(&f.body, lines, source, locator, diags);
                walk_unnecessary_pass(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                check_body_for_unnecessary_pass(&f.body, lines, source, locator, diags);
                walk_unnecessary_pass(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                check_body_for_unnecessary_pass(&c.body, lines, source, locator, diags);
                walk_unnecessary_pass(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                check_body_for_unnecessary_pass(&i.body, lines, source, locator, diags);
                walk_unnecessary_pass(&i.body, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&i.orelse, lines, source, locator, diags);
                walk_unnecessary_pass(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                check_body_for_unnecessary_pass(&f.body, lines, source, locator, diags);
                walk_unnecessary_pass(&f.body, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&f.orelse, lines, source, locator, diags);
                walk_unnecessary_pass(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                check_body_for_unnecessary_pass(&f.body, lines, source, locator, diags);
                walk_unnecessary_pass(&f.body, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&f.orelse, lines, source, locator, diags);
                walk_unnecessary_pass(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                check_body_for_unnecessary_pass(&w.body, lines, source, locator, diags);
                walk_unnecessary_pass(&w.body, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&w.orelse, lines, source, locator, diags);
                walk_unnecessary_pass(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                check_body_for_unnecessary_pass(&w.body, lines, source, locator, diags);
                walk_unnecessary_pass(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                check_body_for_unnecessary_pass(&w.body, lines, source, locator, diags);
                walk_unnecessary_pass(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                check_body_for_unnecessary_pass(&t.body, lines, source, locator, diags);
                walk_unnecessary_pass(&t.body, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&t.orelse, lines, source, locator, diags);
                walk_unnecessary_pass(&t.orelse, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&t.finalbody, lines, source, locator, diags);
                walk_unnecessary_pass(&t.finalbody, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    check_body_for_unnecessary_pass(&h.body, lines, source, locator, diags);
                    walk_unnecessary_pass(&h.body, lines, source, locator, diags);
                }
            }
            ast::Stmt::TryStar(t) => {
                check_body_for_unnecessary_pass(&t.body, lines, source, locator, diags);
                walk_unnecessary_pass(&t.body, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&t.orelse, lines, source, locator, diags);
                walk_unnecessary_pass(&t.orelse, lines, source, locator, diags);
                check_body_for_unnecessary_pass(&t.finalbody, lines, source, locator, diags);
                walk_unnecessary_pass(&t.finalbody, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    check_body_for_unnecessary_pass(&h.body, lines, source, locator, diags);
                    walk_unnecessary_pass(&h.body, lines, source, locator, diags);
                }
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    check_body_for_unnecessary_pass(&case.body, lines, source, locator, diags);
                    walk_unnecessary_pass(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_body_for_unnecessary_pass(
    body: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    // body is only pass → required, skip
    if body.len() == 1 && matches!(body[0], ast::Stmt::Pass(_)) {
        return;
    }
    for stmt in body {
        if let ast::Stmt::Pass(p) = stmt {
            let loc = locator.locate(p.start());
            report(
                loc,
                "readability-unnecessary-pass",
                "Unnecessary pass statement",
                lines,
                source,
                diags,
            );
        }
    }
}

// ── Empty f-string ──────────────────────────────────────────────────────────

fn walk_empty_fstring(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                check_expr_for_empty_fstring(
                    &f.args, lines, source, locator, diags,
                );
                walk_empty_fstring(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                check_expr_for_empty_fstring(
                    &f.args, lines, source, locator, diags,
                );
                walk_empty_fstring(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_empty_fstring(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::Return(r) => {
                if let Some(v) = &r.value {
                    check_expr_tree_for_empty_fstring(v, lines, source, locator, diags);
                }
            }
            ast::Stmt::Assign(a) => {
                check_expr_tree_for_empty_fstring(&a.value, lines, source, locator, diags);
            }
            ast::Stmt::AnnAssign(a) => {
                if let Some(v) = &a.value {
                    check_expr_tree_for_empty_fstring(v, lines, source, locator, diags);
                }
            }
            ast::Stmt::AugAssign(a) => {
                check_expr_tree_for_empty_fstring(&a.value, lines, source, locator, diags);
            }
            ast::Stmt::Expr(e) => {
                check_expr_tree_for_empty_fstring(&e.value, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                check_expr_tree_for_empty_fstring(&i.test, lines, source, locator, diags);
                walk_empty_fstring(&i.body, lines, source, locator, diags);
                walk_empty_fstring(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                check_expr_tree_for_empty_fstring(&w.test, lines, source, locator, diags);
                walk_empty_fstring(&w.body, lines, source, locator, diags);
                walk_empty_fstring(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                check_expr_tree_for_empty_fstring(&f.iter, lines, source, locator, diags);
                walk_empty_fstring(&f.body, lines, source, locator, diags);
                walk_empty_fstring(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                check_expr_tree_for_empty_fstring(&f.iter, lines, source, locator, diags);
                walk_empty_fstring(&f.body, lines, source, locator, diags);
                walk_empty_fstring(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                for item in &w.items {
                    check_expr_tree_for_empty_fstring(
                        &item.context_expr, lines, source, locator, diags,
                    );
                }
                walk_empty_fstring(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                for item in &w.items {
                    check_expr_tree_for_empty_fstring(
                        &item.context_expr, lines, source, locator, diags,
                    );
                }
                walk_empty_fstring(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Assert(a) => {
                check_expr_tree_for_empty_fstring(&a.test, lines, source, locator, diags);
                if let Some(m) = &a.msg {
                    check_expr_tree_for_empty_fstring(m, lines, source, locator, diags);
                }
            }
            ast::Stmt::Raise(r) => {
                if let Some(exc) = &r.exc {
                    check_expr_tree_for_empty_fstring(exc, lines, source, locator, diags);
                }
            }
            ast::Stmt::Try(t) => {
                walk_empty_fstring(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if let Some(ty) = &h.type_ {
                        check_expr_tree_for_empty_fstring(ty, lines, source, locator, diags);
                    }
                    walk_empty_fstring(&h.body, lines, source, locator, diags);
                }
                walk_empty_fstring(&t.orelse, lines, source, locator, diags);
                walk_empty_fstring(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_empty_fstring(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if let Some(ty) = &h.type_ {
                        check_expr_tree_for_empty_fstring(ty, lines, source, locator, diags);
                    }
                    walk_empty_fstring(&h.body, lines, source, locator, diags);
                }
                walk_empty_fstring(&t.orelse, lines, source, locator, diags);
                walk_empty_fstring(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                check_expr_tree_for_empty_fstring(&m.subject, lines, source, locator, diags);
                for case in &m.cases {
                    if let Some(g) = &case.guard {
                        check_expr_tree_for_empty_fstring(g, lines, source, locator, diags);
                    }
                    walk_empty_fstring(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_expr_for_empty_fstring(
    args: &ast::Arguments,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for arg in args.posonlyargs.iter().chain(args.args.iter()).chain(args.kwonlyargs.iter()) {
        if let Some(default) = &arg.default {
            check_expr_tree_for_empty_fstring(default, lines, source, locator, diags);
        }
    }
    // vararg/kwarg don't have defaults in Python
}

fn check_expr_tree_for_empty_fstring(
    expr: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::Expr::JoinedStr(js) => {
            let has_formatted = js
                .values
                .iter()
                .any(|v| matches!(v, ast::Expr::FormattedValue(_)));
            if !has_formatted {
                let loc = locator.locate(js.start());
                report(
                    loc,
                    "readability-empty-fstring",
                    "f-string has no placeholders; use a regular string instead",
                    lines,
                    source,
                    diags,
                );
            }
            // Recurse into values once
            for v in &js.values {
                check_expr_tree_for_empty_fstring(v, lines, source, locator, diags);
            }
            return; // Already recursed, don't walk again
        }
        _ => {}
    }
    walk_expr_for_empty_fstring(expr, lines, source, locator, diags);
}

fn walk_expr_for_empty_fstring(
    expr: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::Expr::BoolOp(b) => {
            for v in &b.values {
                check_expr_tree_for_empty_fstring(v, lines, source, locator, diags);
            }
        }
        ast::Expr::BinOp(b) => {
            check_expr_tree_for_empty_fstring(&b.left, lines, source, locator, diags);
            check_expr_tree_for_empty_fstring(&b.right, lines, source, locator, diags);
        }
        ast::Expr::UnaryOp(u) => {
            check_expr_tree_for_empty_fstring(&u.operand, lines, source, locator, diags);
        }
        ast::Expr::IfExp(i) => {
            check_expr_tree_for_empty_fstring(&i.test, lines, source, locator, diags);
            check_expr_tree_for_empty_fstring(&i.body, lines, source, locator, diags);
            check_expr_tree_for_empty_fstring(&i.orelse, lines, source, locator, diags);
        }
        ast::Expr::Call(c) => {
            check_expr_tree_for_empty_fstring(&c.func, lines, source, locator, diags);
            for arg in &c.args {
                check_expr_tree_for_empty_fstring(arg, lines, source, locator, diags);
            }
            for kw in &c.keywords {
                check_expr_tree_for_empty_fstring(&kw.value, lines, source, locator, diags);
            }
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                check_expr_tree_for_empty_fstring(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                check_expr_tree_for_empty_fstring(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::Set(s) => {
            for elt in &s.elts {
                check_expr_tree_for_empty_fstring(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::Dict(d) => {
            for k in d.keys.iter().flatten() {
                check_expr_tree_for_empty_fstring(k, lines, source, locator, diags);
            }
            for v in &d.values {
                check_expr_tree_for_empty_fstring(v, lines, source, locator, diags);
            }
        }
        ast::Expr::Compare(cmp) => {
            check_expr_tree_for_empty_fstring(&cmp.left, lines, source, locator, diags);
            for c in &cmp.comparators {
                check_expr_tree_for_empty_fstring(c, lines, source, locator, diags);
            }
        }
        ast::Expr::Subscript(s) => {
            check_expr_tree_for_empty_fstring(&s.value, lines, source, locator, diags);
        }
        ast::Expr::Starred(s) => {
            check_expr_tree_for_empty_fstring(&s.value, lines, source, locator, diags);
        }
        ast::Expr::Attribute(a) => {
            check_expr_tree_for_empty_fstring(&a.value, lines, source, locator, diags);
        }
        ast::Expr::FormattedValue(f) => {
            check_expr_tree_for_empty_fstring(&f.value, lines, source, locator, diags);
            // Skip format_spec: it's an internal JoinedStr, not a user-written f-string
        }
        _ => {}
    }
}

// ── Redundant if-return (bool) ──────────────────────────────────────────────

fn walk_simplify_if_return(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                walk_simplify_if_return(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                walk_simplify_if_return(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_simplify_if_return(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                check_if_return_bool(i, lines, source, locator, diags);
                walk_simplify_if_return(&i.body, lines, source, locator, diags);
                walk_simplify_if_return(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                walk_simplify_if_return(&f.body, lines, source, locator, diags);
                walk_simplify_if_return(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_simplify_if_return(&f.body, lines, source, locator, diags);
                walk_simplify_if_return(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_simplify_if_return(&w.body, lines, source, locator, diags);
                walk_simplify_if_return(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_simplify_if_return(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_simplify_if_return(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_simplify_if_return(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_simplify_if_return(&h.body, lines, source, locator, diags);
                }
                walk_simplify_if_return(&t.orelse, lines, source, locator, diags);
                walk_simplify_if_return(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_simplify_if_return(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_simplify_if_return(&h.body, lines, source, locator, diags);
                }
                walk_simplify_if_return(&t.orelse, lines, source, locator, diags);
                walk_simplify_if_return(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_simplify_if_return(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_if_return_bool(
    if_node: &ast::StmtIf,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    // Pattern: if cond: return True \n else: return False
    // Or:     if cond: return False \n else: return True
    if if_node.body.len() == 1 && if_node.orelse.len() == 1 {
        let body_ret = match &if_node.body[0] {
            ast::Stmt::Return(r) => r,
            _ => return,
        };
        let else_ret = match &if_node.orelse[0] {
            ast::Stmt::Return(r) => r,
            _ => return,
        };
        let (Some(b), Some(e)) = (
            extract_bool(body_ret.value.as_deref()),
            extract_bool(else_ret.value.as_deref()),
        ) else {
            return;
        };
        if b != e {
            let loc = locator.locate(if_node.start());
            report(
                loc,
                "readability-simplify-if-return",
                "Redundant if-return; simplify to a single return expression",
                lines,
                source,
                diags,
            );
        }
    }
}

fn extract_bool(expr: Option<&ast::Expr>) -> Option<bool> {
    expr.and_then(|e| {
        if let ast::Expr::Constant(c) = e {
            if let ast::Constant::Bool(b) = c.value {
                return Some(b);
            }
        }
        None
    })
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

    // ── Unnecessary pass ─────────────────────────────────────────────────

    #[test]
    fn unnecessary_pass_with_docstring() {
        let src = "def foo():\n    \"\"\"Doc.\"\"\"\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn pass_only_ok() {
        let src = "def foo():\n    pass\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn pass_with_statement() {
        let src = "def foo():\n    x = 1\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn unnecessary_pass_in_class() {
        let src = "class Foo:\n    \"\"\"Doc.\"\"\"\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn unnecessary_pass_in_if() {
        let src = "if True:\n    x = 1\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn unnecessary_pass_in_try_except() {
        let src = "try:\n    x = 1\n    pass\nexcept:\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn unnecessary_pass_in_for() {
        let src = "for x in y:\n    z = 1\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn unnecessary_pass_in_while() {
        let src = "while True:\n    x = 1\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn pass_only_in_if_ok() {
        let src = "if True:\n    pass\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    #[test]
    fn unnecessary_pass_in_match_case() {
        let src = "match x:\n    case 1:\n        y = 1\n        pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
    }

    // ── Empty f-string ───────────────────────────────────────────────────

    #[test]
    fn empty_fstring() {
        let src = "x = f\"hello\"\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    #[test]
    fn fstring_with_placeholder_ok() {
        let src = "x = f\"hello {name}\"\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    #[test]
    fn empty_fstring_in_function() {
        let src = "def foo():\n    x = f\"hello\"\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    #[test]
    fn fstring_in_if_condition() {
        let src = "if f\"test\":\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    #[test]
    fn fstring_with_format_spec_ok() {
        let src = "x = f\"{name:.2f}\"\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    #[test]
    fn empty_fstring_in_assignment() {
        let src = "x: str = f\"test\"\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    // ── Simplify if-return ───────────────────────────────────────────────

    #[test]
    fn simplify_if_return_bool() {
        let src = "def foo(x):\n    if x:\n        return True\n    else:\n        return False\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn simplify_if_return_bool_inverted() {
        let src = "def foo(x):\n    if x:\n        return False\n    else:\n        return True\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn normal_if_return_ok() {
        let src = "def foo(x):\n    if x:\n        return compute()\n    return 0\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn if_return_same_bool_ok() {
        let src = "def foo(x):\n    if x:\n        return True\n    else:\n        return True\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn if_return_non_bool_expr_ok() {
        let src = "def foo(x):\n    if x:\n        return 1\n    else:\n        return 0\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn if_return_no_else_ok() {
        let src = "def foo(x):\n    if x:\n        return True\n    return False\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn empty_fstring_nested() {
        let src = "x = f\"{f'hello'}\"\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-empty-fstring"));
    }

    #[test]
    fn if_elif_return_no_false_positive() {
        let src = "def foo(x, y):\n    if x:\n        return True\n    elif y:\n        return False\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    #[test]
    fn bare_return_no_false_positive() {
        let src = "def foo(x):\n    if x:\n        return\n    else:\n        return\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"readability-simplify-if-return"));
    }

    // ── Integration ──────────────────────────────────────────────────────

    #[test]
    fn empty_source_no_diags() {
        let diags = analyze("");
        assert!(diags.is_empty());
    }

    #[test]
    fn clean_code_no_diags() {
        let src = "def foo(x):\n    return x + 1\n";
        let diags = analyze(src);
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_strict_issues() {
        let src = "def foo():\n    \"\"\"Doc.\"\"\"\n    pass\n\nx = f\"hello\"\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"readability-unnecessary-pass"));
        assert!(rule_ids(&diags).contains(&"readability-empty-fstring"));
    }
}
