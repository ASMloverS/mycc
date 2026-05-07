use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::MagicNumberConfig;
use num_traits::cast::ToPrimitive;
use rustpython_parser::ast::{self, Ranged};
use std::collections::HashSet;

pub fn check_magic_number(source: &SourceFile, config: &MagicNumberConfig) -> Vec<Diagnostic> {
    if !config.enabled {
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
    let allowed: HashSet<i64> = config.allowed.iter().copied().collect();

    let body = match &ast {
        ast::Mod::Module(m) => &m.body,
        _ => return diags,
    };

    let mut ctx = WalkContext {
        lines: &lines,
        source,
        locator,
        allowed: &allowed,
        diags: &mut diags,
    };

    walk_stmts(body, &mut ctx, ContextKind::General);

    diags
}

struct WalkContext<'a> {
    lines: &'a [&'a str],
    source: &'a SourceFile,
    locator: rustpython_parser::source_code::RandomLocator<'a>,
    allowed: &'a HashSet<i64>,
    diags: &'a mut Vec<Diagnostic>,
}

#[derive(Clone, Copy, PartialEq)]
enum ContextKind {
    General,
    RangeCall,
    Slice,
    ConstAssign,
    VersionAssign,
    DefaultParam,
    Annotation,
}

fn walk_stmts(stmts: &[ast::Stmt], ctx: &mut WalkContext, kind: ContextKind) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                for dec in &f.decorator_list {
                    walk_expr(dec, ctx, ContextKind::General);
                }
                walk_args(&f.args, ctx);
                walk_stmts(&f.body, ctx, ContextKind::General);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                for dec in &f.decorator_list {
                    walk_expr(dec, ctx, ContextKind::General);
                }
                walk_args(&f.args, ctx);
                walk_stmts(&f.body, ctx, ContextKind::General);
            }
            ast::Stmt::ClassDef(c) => {
                walk_stmts(&c.body, ctx, ContextKind::General);
            }
            ast::Stmt::Assign(a) => {
                let is_const = a.targets.len() == 1
                    && matches!(
                        &a.targets[0],
                        ast::Expr::Name(n) if is_upper_snake_case(n.id.as_str())
                    );
                let is_version = a.targets.len() == 1
                    && matches!(
                        &a.targets[0],
                        ast::Expr::Name(n) if n.id.as_str() == "__version__"
                    );
                let assign_kind = if is_version {
                    ContextKind::VersionAssign
                } else if is_const {
                    ContextKind::ConstAssign
                } else {
                    kind
                };
                walk_expr(&a.value, ctx, assign_kind);
            }
            ast::Stmt::AnnAssign(a) => {
                walk_expr(&a.annotation, ctx, ContextKind::Annotation);
                if let Some(ref value) = a.value {
                    walk_expr(value, ctx, kind);
                }
            }
            ast::Stmt::Return(r) => {
                if let Some(ref value) = r.value {
                    walk_expr(value, ctx, kind);
                }
            }
            ast::Stmt::For(f) => {
                walk_expr(&f.target, ctx, kind);
                walk_expr(&f.iter, ctx, kind);
                walk_stmts(&f.body, ctx, kind);
                walk_stmts(&f.orelse, ctx, kind);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_expr(&f.target, ctx, kind);
                walk_expr(&f.iter, ctx, kind);
                walk_stmts(&f.body, ctx, kind);
                walk_stmts(&f.orelse, ctx, kind);
            }
            ast::Stmt::While(w) => {
                walk_expr(&w.test, ctx, kind);
                walk_stmts(&w.body, ctx, kind);
                walk_stmts(&w.orelse, ctx, kind);
            }
            ast::Stmt::If(i) => {
                walk_expr(&i.test, ctx, kind);
                walk_stmts(&i.body, ctx, kind);
                walk_stmts(&i.orelse, ctx, kind);
            }
            ast::Stmt::With(w) => {
                for item in &w.items {
                    walk_expr(&item.context_expr, ctx, kind);
                    if let Some(ref vars) = item.optional_vars {
                        walk_expr(vars, ctx, kind);
                    }
                }
                walk_stmts(&w.body, ctx, kind);
            }
            ast::Stmt::AsyncWith(w) => {
                for item in &w.items {
                    walk_expr(&item.context_expr, ctx, kind);
                    if let Some(ref vars) = item.optional_vars {
                        walk_expr(vars, ctx, kind);
                    }
                }
                walk_stmts(&w.body, ctx, kind);
            }
            ast::Stmt::Raise(r) => {
                if let Some(ref exc) = r.exc {
                    walk_expr(exc, ctx, kind);
                }
                if let Some(ref cause) = r.cause {
                    walk_expr(cause, ctx, kind);
                }
            }
            ast::Stmt::Assert(a) => {
                walk_expr(&a.test, ctx, kind);
                if let Some(ref msg) = a.msg {
                    walk_expr(msg, ctx, kind);
                }
            }
            ast::Stmt::Expr(e) => {
                walk_expr(&e.value, ctx, kind);
            }
            ast::Stmt::Try(t) => {
                walk_stmts(&t.body, ctx, kind);
                walk_handlers(&t.handlers, ctx, kind);
                walk_stmts(&t.orelse, ctx, kind);
                walk_stmts(&t.finalbody, ctx, kind);
            }
            ast::Stmt::TryStar(t) => {
                walk_stmts(&t.body, ctx, kind);
                walk_handlers(&t.handlers, ctx, kind);
                walk_stmts(&t.orelse, ctx, kind);
                walk_stmts(&t.finalbody, ctx, kind);
            }
            ast::Stmt::Match(m) => {
                walk_expr(&m.subject, ctx, kind);
                for case in &m.cases {
                    if let Some(ref guard) = case.guard {
                        walk_expr(guard, ctx, kind);
                    }
                    walk_stmts(&case.body, ctx, kind);
                }
            }
            ast::Stmt::Import(_) | ast::Stmt::ImportFrom(_) => {}
            ast::Stmt::Global(_) | ast::Stmt::Nonlocal(_) | ast::Stmt::Pass(_) => {}
            ast::Stmt::Break(_) | ast::Stmt::Continue(_) => {}
            ast::Stmt::Delete(d) => {
                for target in &d.targets {
                    walk_expr(target, ctx, kind);
                }
            }
            ast::Stmt::AugAssign(a) => {
                walk_expr(&a.target, ctx, kind);
                walk_expr(&a.value, ctx, kind);
            }
            ast::Stmt::TypeAlias(t) => {
                walk_expr(&t.name, ctx, ContextKind::Annotation);
                walk_expr(&t.value, ctx, ContextKind::Annotation);
            }
        }
    }
}

fn walk_handlers(handlers: &[ast::ExceptHandler], ctx: &mut WalkContext, kind: ContextKind) {
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        if let Some(ref exc) = h.type_ {
            walk_expr(exc, ctx, kind);
        }
        walk_stmts(&h.body, ctx, kind);
    }
}

fn walk_args(args: &ast::Arguments, ctx: &mut WalkContext) {
    for arg in &args.posonlyargs {
        if let Some(ref default) = arg.default {
            walk_expr(default, ctx, ContextKind::DefaultParam);
        }
    }
    for arg in &args.args {
        if let Some(ref default) = arg.default {
            walk_expr(default, ctx, ContextKind::DefaultParam);
        }
    }
    for arg in &args.kwonlyargs {
        if let Some(ref default) = arg.default {
            walk_expr(default, ctx, ContextKind::DefaultParam);
        }
    }
}

fn walk_expr(expr: &ast::Expr, ctx: &mut WalkContext, kind: ContextKind) {
    match expr {
        ast::Expr::Constant(c) => {
            match &c.value {
                ast::Constant::Int(i) => {
                    let val = i.to_i64();
                    if let Some(val) = val {
                        check_number(val, c.start(), ctx, kind);
                    }
                }
                ast::Constant::Float(f) => {
                    check_float(*f, c.start(), ctx, kind);
                }
                _ => {}
            }
        }
        ast::Expr::UnaryOp(u) => {
            if matches!(u.op, ast::UnaryOp::USub) {
                if let ast::Expr::Constant(c) = u.operand.as_ref() {
                    if let ast::Constant::Int(i) = &c.value {
                        let val = i.to_i64();
                        if let Some(val) = val {
                            check_number(-val, u.start(), ctx, kind);
                            return;
                        }
                    }
                    if let ast::Constant::Float(f) = &c.value {
                        check_float(-*f, u.start(), ctx, kind);
                        return;
                    }
                }
            }
            walk_expr(&u.operand, ctx, kind);
        }
        ast::Expr::BinOp(b) => {
            walk_expr(&b.left, ctx, kind);
            walk_expr(&b.right, ctx, kind);
        }
        ast::Expr::BoolOp(b) => {
            for v in &b.values {
                walk_expr(v, ctx, kind);
            }
        }
        ast::Expr::Compare(c) => {
            walk_expr(&c.left, ctx, kind);
            for comp in &c.comparators {
                walk_expr(comp, ctx, kind);
            }
        }
        ast::Expr::Call(c) => {
            let is_range = matches!(
                c.func.as_ref(),
                ast::Expr::Name(n) if n.id.as_str() == "range"
            );
            let call_kind = if is_range {
                ContextKind::RangeCall
            } else {
                kind
            };
            walk_expr(&c.func, ctx, kind);
            for arg in &c.args {
                walk_expr(arg, ctx, call_kind);
            }
            for kw in &c.keywords {
                walk_expr(&kw.value, ctx, call_kind);
            }
        }
        ast::Expr::Subscript(s) => {
            walk_expr(&s.value, ctx, kind);
            walk_expr(&s.slice, ctx, ContextKind::Slice);
        }
        ast::Expr::Slice(s) => {
            if let Some(ref lower) = s.lower {
                walk_expr(lower, ctx, ContextKind::Slice);
            }
            if let Some(ref upper) = s.upper {
                walk_expr(upper, ctx, ContextKind::Slice);
            }
            if let Some(ref step) = s.step {
                walk_expr(step, ctx, ContextKind::Slice);
            }
        }
        ast::Expr::IfExp(i) => {
            walk_expr(&i.test, ctx, kind);
            walk_expr(&i.body, ctx, kind);
            walk_expr(&i.orelse, ctx, kind);
        }
        ast::Expr::Lambda(l) => {
            walk_args(&l.args, ctx);
            walk_expr(&l.body, ctx, kind);
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                walk_expr(elt, ctx, kind);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                walk_expr(elt, ctx, kind);
            }
        }
        ast::Expr::Set(s) => {
            for elt in &s.elts {
                walk_expr(elt, ctx, kind);
            }
        }
        ast::Expr::Dict(d) => {
            for (k, v) in d.keys.iter().zip(&d.values) {
                if let Some(key) = k {
                    walk_expr(key, ctx, kind);
                }
                walk_expr(v, ctx, kind);
            }
        }
        ast::Expr::NamedExpr(n) => {
            walk_expr(&n.target, ctx, kind);
            walk_expr(&n.value, ctx, kind);
        }
        ast::Expr::Attribute(a) => {
            walk_expr(&a.value, ctx, kind);
        }
        ast::Expr::Starred(s) => {
            walk_expr(&s.value, ctx, kind);
        }
        ast::Expr::ListComp(l) => {
            walk_expr(&l.elt, ctx, kind);
            for gen in &l.generators {
                walk_comprehension(gen, ctx, kind);
            }
        }
        ast::Expr::SetComp(s) => {
            walk_expr(&s.elt, ctx, kind);
            for gen in &s.generators {
                walk_comprehension(gen, ctx, kind);
            }
        }
        ast::Expr::DictComp(d) => {
            walk_expr(&d.key, ctx, kind);
            walk_expr(&d.value, ctx, kind);
            for gen in &d.generators {
                walk_comprehension(gen, ctx, kind);
            }
        }
        ast::Expr::GeneratorExp(g) => {
            walk_expr(&g.elt, ctx, kind);
            for gen in &g.generators {
                walk_comprehension(gen, ctx, kind);
            }
        }
        ast::Expr::Await(a) => {
            walk_expr(&a.value, ctx, kind);
        }
        ast::Expr::Yield(y) => {
            if let Some(ref value) = y.value {
                walk_expr(value, ctx, kind);
            }
        }
        ast::Expr::YieldFrom(y) => {
            walk_expr(&y.value, ctx, kind);
        }
        ast::Expr::JoinedStr(j) => {
            for value in &j.values {
                walk_expr(value, ctx, kind);
            }
        }
        ast::Expr::FormattedValue(f) => {
            walk_expr(&f.value, ctx, kind);
            if let Some(ref spec) = f.format_spec {
                walk_expr(spec, ctx, kind);
            }
        }
        ast::Expr::Name(_) => {}
    }
}

fn walk_comprehension(comp: &ast::Comprehension, ctx: &mut WalkContext, kind: ContextKind) {
    walk_expr(&comp.target, ctx, kind);
    walk_expr(&comp.iter, ctx, kind);
    for if_ in &comp.ifs {
        walk_expr(if_, ctx, kind);
    }
}

fn is_excluded_kind(kind: ContextKind) -> bool {
    matches!(
        kind,
        ContextKind::RangeCall
            | ContextKind::Slice
            | ContextKind::ConstAssign
            | ContextKind::VersionAssign
            | ContextKind::DefaultParam
            | ContextKind::Annotation
    )
}

fn emit_diag(val: impl std::fmt::Display, pos: ast::TextSize, ctx: &mut WalkContext) {
    let line = ctx.locator.locate(pos).row.to_usize();
    let source_line = ctx.lines.get(line - 1).copied().unwrap_or("");
    ctx.diags.push(Diagnostic::new_with_source(
        ctx.source.display_path(),
        line,
        1,
        Severity::Warning,
        "readability-magic-number",
        &format!("Magic number: {}", val),
        source_line,
    ));
}

fn check_number(val: i64, pos: ast::TextSize, ctx: &mut WalkContext, kind: ContextKind) {
    if !is_excluded_kind(kind) && !ctx.allowed.contains(&val) {
        emit_diag(val, pos, ctx);
    }
}

fn check_float(val: f64, pos: ast::TextSize, ctx: &mut WalkContext, kind: ContextKind) {
    if !is_excluded_kind(kind) {
        emit_diag(val, pos, ctx);
    }
}

fn is_upper_snake_case(name: &str) -> bool {
    let mut has_upper = false;
    for c in name.chars() {
        match c {
            'A'..='Z' => has_upper = true,
            '_' | '0'..='9' => {}
            _ => return false,
        }
    }
    has_upper
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn check(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_magic_number(&source, &MagicNumberConfig::default())
    }

    fn rule_ids(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule_id.as_str()).collect()
    }

    #[test]
    fn detect_magic_number() {
        let diags = check("x = calculate(42)\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn allowed_numbers_no_warning() {
        let diags = check("x = y + 1\nz = w - 0\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn constant_definition_no_warning() {
        let diags = check("MAX_SIZE = 100\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn default_param_no_warning() {
        let diags = check("def foo(timeout=30):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn range_no_warning() {
        let diags = check("for i in range(10):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn negative_magic_number() {
        let diags = check("x = calculate(-42)\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn allowed_negative_one() {
        let diags = check("x = y + (-1)\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn slice_no_warning() {
        let diags = check("x = arr[1:10:2]\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn version_no_warning() {
        let diags = check("__version__ = \"1.2.3\"\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn disabled_config() {
        let source = SourceFile::from_string("x = calculate(42)\n", PathBuf::from("test.py"));
        let config = MagicNumberConfig {
            enabled: false,
            allowed: vec![0, 1, -1, 2],
        };
        let diags = check_magic_number(&source, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn custom_allowed() {
        let source = SourceFile::from_string("x = calculate(42)\n", PathBuf::from("test.py"));
        let config = MagicNumberConfig {
            enabled: true,
            allowed: vec![0, 1, -1, 2, 42],
        };
        let diags = check_magic_number(&source, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn parse_error_returns_empty() {
        let diags = check("def foo(:\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_magic_numbers() {
        let diags = check("x = 42\ny = 99\n");
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn non_const_lower_assign_flags() {
        let diags = check("my_var = 42\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn underscore_prefix_const_no_warning() {
        let diags = check("_MY_CONST = 42\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn const_assign_with_allowed_value() {
        let diags = check("MAX_SIZE = 1\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn function_body_magic_number() {
        let diags = check("def foo():\n    x = 42\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn nested_call_magic_number() {
        let diags = check("x = foo(bar(42))\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn range_with_step_no_warning() {
        let diags = check("for i in range(0, 10, 2):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn keyword_range_no_warning() {
        let diags = check("for i in range(stop=10):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn binop_magic_number() {
        let diags = check("x = y + 42\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn diagnostic_contains_source_line() {
        let diags = check("x = calculate(42)\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].source_line.as_deref(), Some("x = calculate(42)"));
    }

    #[test]
    fn rule_id_has_readability_prefix() {
        let diags = check("x = calculate(42)\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].rule_id.starts_with("readability-"));
    }

    #[test]
    fn async_function_default_param_no_warning() {
        let diags = check("async def foo(timeout=30):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn class_body_magic_number() {
        let diags = check("class Foo:\n    x = 42\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn class_const_no_warning() {
        let diags = check("class Foo:\n    MAX = 100\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn allowed_two() {
        let diags = check("x = y * 2\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn kwonly_default_param_no_warning() {
        let diags = check("def foo(*, timeout=30):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn posonly_default_param_no_warning() {
        let diags = check("def foo(timeout=30, /):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn float_literal_detected() {
        let diags = check("x = calculate(3.14)\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
        assert!(diags[0].message.contains("3.14"));
    }

    #[test]
    fn negative_float_literal_detected() {
        let diags = check("x = calculate(-3.14)\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
        assert!(diags[0].message.contains("-3.14"));
    }

    #[test]
    fn underscore_var_not_treated_as_const() {
        let diags = check("_temp_value = 42\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn upper_snake_const_still_excluded() {
        let diags = check("MY_CONST = 42\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn decorator_args_checked() {
        let diags = check("@retry(3)\ndef foo():\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-magic-number"]);
    }

    #[test]
    fn float_const_assign_no_warning() {
        let diags = check("PI = 3.14\n");
        assert!(diags.is_empty());
    }
}
