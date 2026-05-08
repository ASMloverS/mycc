use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::ProhibitedConfig;
use rustpython_parser::ast::{self, Ranged};
use std::collections::HashSet;

fn default_prohibited() -> Vec<String> {
    vec![
        "eval".into(),
        "exec".into(),
        "__import__".into(),
        "os.system".into(),
        "subprocess.call".into(),
    ]
}

pub fn check_prohibited(source: &SourceFile, config: &ProhibitedConfig) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    let Ok(ast) =
        rustpython_parser::parse(&source.content, rustpython_parser::Mode::Module, "<input>")
    else {
        return diags;
    };

    let mut prohibited: HashSet<String> = HashSet::new();
    if config.use_default {
        prohibited.extend(default_prohibited());
    }
    prohibited.extend(config.extra.iter().cloned());
    for remove in &config.remove {
        prohibited.remove(remove);
    }

    if prohibited.is_empty() {
        return diags;
    }

    let lines = source.lines();
    let mut locator = rustpython_parser::source_code::RandomLocator::new(&source.content);

    let body = match &ast {
        ast::Mod::Module(m) => &m.body,
        _ => return diags,
    };

    walk_stmts(body, &lines, source, &mut locator, &prohibited, &mut diags);

    diags
}

fn walk_stmts(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    prohibited: &HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                for dec in &f.decorator_list {
                    walk_expr(dec, lines, source, locator, prohibited, diags);
                }
                walk_args(&f.args, lines, source, locator, prohibited, diags);
                if let Some(ref ret) = f.returns {
                    walk_expr(ret, lines, source, locator, prohibited, diags);
                }
                walk_stmts(&f.body, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                for dec in &f.decorator_list {
                    walk_expr(dec, lines, source, locator, prohibited, diags);
                }
                walk_args(&f.args, lines, source, locator, prohibited, diags);
                if let Some(ref ret) = f.returns {
                    walk_expr(ret, lines, source, locator, prohibited, diags);
                }
                walk_stmts(&f.body, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::ClassDef(c) => {
                for base in &c.bases {
                    walk_expr(base, lines, source, locator, prohibited, diags);
                }
                for kw in &c.keywords {
                    walk_expr(&kw.value, lines, source, locator, prohibited, diags);
                }
                for dec in &c.decorator_list {
                    walk_expr(dec, lines, source, locator, prohibited, diags);
                }
                walk_stmts(&c.body, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::Return(r) => {
                if let Some(ref value) = r.value {
                    walk_expr(value, lines, source, locator, prohibited, diags);
                }
            }
            ast::Stmt::Assign(a) => {
                for target in &a.targets {
                    walk_expr(target, lines, source, locator, prohibited, diags);
                }
                walk_expr(&a.value, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::AugAssign(a) => {
                walk_expr(&a.target, lines, source, locator, prohibited, diags);
                walk_expr(&a.value, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::AnnAssign(a) => {
                walk_expr(&a.annotation, lines, source, locator, prohibited, diags);
                if let Some(ref value) = a.value {
                    walk_expr(value, lines, source, locator, prohibited, diags);
                }
            }
            ast::Stmt::For(f) => {
                walk_expr(&f.iter, lines, source, locator, prohibited, diags);
                walk_stmts(&f.body, lines, source, locator, prohibited, diags);
                walk_stmts(&f.orelse, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_expr(&f.iter, lines, source, locator, prohibited, diags);
                walk_stmts(&f.body, lines, source, locator, prohibited, diags);
                walk_stmts(&f.orelse, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::While(w) => {
                walk_expr(&w.test, lines, source, locator, prohibited, diags);
                walk_stmts(&w.body, lines, source, locator, prohibited, diags);
                walk_stmts(&w.orelse, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::If(i) => {
                walk_expr(&i.test, lines, source, locator, prohibited, diags);
                walk_stmts(&i.body, lines, source, locator, prohibited, diags);
                walk_stmts(&i.orelse, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::With(w) => {
                for item in &w.items {
                    walk_expr(&item.context_expr, lines, source, locator, prohibited, diags);
                    if let Some(ref vars) = item.optional_vars {
                        walk_expr(vars, lines, source, locator, prohibited, diags);
                    }
                }
                walk_stmts(&w.body, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                for item in &w.items {
                    walk_expr(&item.context_expr, lines, source, locator, prohibited, diags);
                    if let Some(ref vars) = item.optional_vars {
                        walk_expr(vars, lines, source, locator, prohibited, diags);
                    }
                }
                walk_stmts(&w.body, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::Raise(r) => {
                if let Some(ref exc) = r.exc {
                    walk_expr(exc, lines, source, locator, prohibited, diags);
                }
                if let Some(ref cause) = r.cause {
                    walk_expr(cause, lines, source, locator, prohibited, diags);
                }
            }
            ast::Stmt::Assert(a) => {
                walk_expr(&a.test, lines, source, locator, prohibited, diags);
                if let Some(ref msg) = a.msg {
                    walk_expr(msg, lines, source, locator, prohibited, diags);
                }
            }
            ast::Stmt::Try(t) => {
                walk_stmts(&t.body, lines, source, locator, prohibited, diags);
                walk_handlers(&t.handlers, lines, source, locator, prohibited, diags);
                walk_stmts(&t.orelse, lines, source, locator, prohibited, diags);
                walk_stmts(&t.finalbody, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_stmts(&t.body, lines, source, locator, prohibited, diags);
                walk_handlers(&t.handlers, lines, source, locator, prohibited, diags);
                walk_stmts(&t.orelse, lines, source, locator, prohibited, diags);
                walk_stmts(&t.finalbody, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::Match(m) => {
                walk_expr(&m.subject, lines, source, locator, prohibited, diags);
                for case in &m.cases {
                    if let Some(ref guard) = case.guard {
                        walk_expr(guard, lines, source, locator, prohibited, diags);
                    }
                    walk_stmts(&case.body, lines, source, locator, prohibited, diags);
                }
            }
            ast::Stmt::Expr(e) => {
                walk_expr(&e.value, lines, source, locator, prohibited, diags);
            }
            ast::Stmt::Delete(d) => {
                for target in &d.targets {
                    walk_expr(target, lines, source, locator, prohibited, diags);
                }
            }
            ast::Stmt::TypeAlias(t) => {
                walk_expr(&t.value, lines, source, locator, prohibited, diags);
            }
            _ => {}
        }
    }
}

fn walk_handlers(
    handlers: &[ast::ExceptHandler],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    prohibited: &HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        if let Some(ref exc) = h.type_ {
            walk_expr(exc, lines, source, locator, prohibited, diags);
        }
        walk_stmts(&h.body, lines, source, locator, prohibited, diags);
    }
}

fn walk_args(
    args: &ast::Arguments,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    prohibited: &HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    for arg in args.posonlyargs.iter().chain(&args.args).chain(&args.kwonlyargs) {
        if let Some(ref ann) = arg.def.annotation {
            walk_expr(ann, lines, source, locator, prohibited, diags);
        }
        if let Some(ref default) = arg.default {
            walk_expr(default, lines, source, locator, prohibited, diags);
        }
    }
    for arg in args.vararg.iter().chain(args.kwarg.iter()) {
        if let Some(ref ann) = arg.annotation {
            walk_expr(ann, lines, source, locator, prohibited, diags);
        }
    }
}

fn walk_expr(
    expr: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    prohibited: &HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    match expr {
        ast::Expr::Call(c) => {
            let call_path = get_call_path(&c.func);
            if prohibited.contains(&call_path) {
                let should_flag =
                    call_path != "subprocess.call" || !has_shell_false(&c.keywords);
                if should_flag {
                    let line = locator.locate(c.start()).row.to_usize();
                    let source_line = lines.get(line - 1).copied().unwrap_or("");
                    diags.push(Diagnostic::new_with_source(
                        source.display_path(),
                        line,
                        1,
                        Severity::Warning,
                        "readability-prohibited",
                        &format!("Prohibited function call: {}", call_path),
                        source_line,
                    ));
                }
            }
            walk_expr(&c.func, lines, source, locator, prohibited, diags);
            for arg in &c.args {
                walk_expr(arg, lines, source, locator, prohibited, diags);
            }
            for kw in &c.keywords {
                walk_expr(&kw.value, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::BinOp(b) => {
            walk_expr(&b.left, lines, source, locator, prohibited, diags);
            walk_expr(&b.right, lines, source, locator, prohibited, diags);
        }
        ast::Expr::UnaryOp(u) => {
            walk_expr(&u.operand, lines, source, locator, prohibited, diags);
        }
        ast::Expr::BoolOp(b) => {
            for v in &b.values {
                walk_expr(v, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Compare(c) => {
            walk_expr(&c.left, lines, source, locator, prohibited, diags);
            for comp in &c.comparators {
                walk_expr(comp, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Subscript(s) => {
            walk_expr(&s.value, lines, source, locator, prohibited, diags);
            walk_expr(&s.slice, lines, source, locator, prohibited, diags);
        }
        ast::Expr::Slice(s) => {
            if let Some(ref lower) = s.lower {
                walk_expr(lower, lines, source, locator, prohibited, diags);
            }
            if let Some(ref upper) = s.upper {
                walk_expr(upper, lines, source, locator, prohibited, diags);
            }
            if let Some(ref step) = s.step {
                walk_expr(step, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                walk_expr(elt, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                walk_expr(elt, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Set(s) => {
            for elt in &s.elts {
                walk_expr(elt, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Dict(d) => {
            for (k, v) in d.keys.iter().zip(&d.values) {
                if let Some(key) = k {
                    walk_expr(key, lines, source, locator, prohibited, diags);
                }
                walk_expr(v, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::IfExp(i) => {
            walk_expr(&i.test, lines, source, locator, prohibited, diags);
            walk_expr(&i.body, lines, source, locator, prohibited, diags);
            walk_expr(&i.orelse, lines, source, locator, prohibited, diags);
        }
        ast::Expr::Lambda(l) => {
            walk_args(&l.args, lines, source, locator, prohibited, diags);
            walk_expr(&l.body, lines, source, locator, prohibited, diags);
        }
        ast::Expr::NamedExpr(n) => {
            walk_expr(&n.value, lines, source, locator, prohibited, diags);
        }
        ast::Expr::Attribute(a) => {
            walk_expr(&a.value, lines, source, locator, prohibited, diags);
        }
        ast::Expr::Starred(s) => {
            walk_expr(&s.value, lines, source, locator, prohibited, diags);
        }
        ast::Expr::ListComp(l) => {
            walk_expr(&l.elt, lines, source, locator, prohibited, diags);
            for gen in &l.generators {
                walk_comprehension(gen, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::SetComp(s) => {
            walk_expr(&s.elt, lines, source, locator, prohibited, diags);
            for gen in &s.generators {
                walk_comprehension(gen, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::DictComp(d) => {
            walk_expr(&d.key, lines, source, locator, prohibited, diags);
            walk_expr(&d.value, lines, source, locator, prohibited, diags);
            for gen in &d.generators {
                walk_comprehension(gen, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::GeneratorExp(g) => {
            walk_expr(&g.elt, lines, source, locator, prohibited, diags);
            for gen in &g.generators {
                walk_comprehension(gen, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Await(a) => {
            walk_expr(&a.value, lines, source, locator, prohibited, diags);
        }
        ast::Expr::Yield(y) => {
            if let Some(ref value) = y.value {
                walk_expr(value, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::YieldFrom(y) => {
            walk_expr(&y.value, lines, source, locator, prohibited, diags);
        }
        ast::Expr::JoinedStr(j) => {
            for value in &j.values {
                walk_expr(value, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::FormattedValue(f) => {
            walk_expr(&f.value, lines, source, locator, prohibited, diags);
            if let Some(ref spec) = f.format_spec {
                walk_expr(spec, lines, source, locator, prohibited, diags);
            }
        }
        ast::Expr::Name(_) | ast::Expr::Constant(_) => {}
    }
}

fn walk_comprehension(
    comp: &ast::Comprehension,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    prohibited: &HashSet<String>,
    diags: &mut Vec<Diagnostic>,
) {
    walk_expr(&comp.iter, lines, source, locator, prohibited, diags);
    for if_ in &comp.ifs {
        walk_expr(if_, lines, source, locator, prohibited, diags);
    }
}

fn has_shell_false(keywords: &[ast::Keyword]) -> bool {
    keywords.iter().any(|kw| {
        kw.arg
            .as_ref()
            .map_or(false, |a| a.as_str() == "shell")
            && matches!(
                &kw.value,
                ast::Expr::Constant(c) if matches!(c.value, ast::Constant::Bool(false))
            )
    })
}

fn get_call_path(func: &ast::Expr) -> String {
    match func {
        ast::Expr::Name(n) => n.id.to_string(),
        ast::Expr::Attribute(a) => {
            let base = get_call_path(&a.value);
            format!("{}.{}", base, a.attr)
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn check(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_prohibited(&source, &ProhibitedConfig::default())
    }

    fn check_with_config(src: &str, config: &ProhibitedConfig) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_prohibited(&source, config)
    }

    fn rule_ids(diags: &[Diagnostic]) -> Vec<&str> {
        diags.iter().map(|d| d.rule_id.as_str()).collect()
    }

    #[test]
    fn detect_eval() {
        let diags = check("x = eval('1+1')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn detect_exec() {
        let diags = check("exec('print(1)')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("exec"));
    }

    #[test]
    fn detect_dunder_import() {
        let diags = check("x = __import__('os')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("__import__"));
    }

    #[test]
    fn detect_os_system() {
        let diags = check("import os\nos.system('ls')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("os.system"));
    }

    #[test]
    fn no_warning_for_allowed() {
        let diags = check("x = int('42')\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn no_warning_for_print() {
        let diags = check("print('hello')\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn custom_prohibited() {
        let config = ProhibitedConfig {
            use_default: false,
            extra: vec!["print".into()],
            remove: vec![],
        };
        let diags = check_with_config("print('hello')\n", &config);
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("print"));
    }

    #[test]
    fn remove_default_prohibited() {
        let config = ProhibitedConfig {
            use_default: true,
            extra: vec![],
            remove: vec!["eval".into()],
        };
        let diags = check_with_config("x = eval('1+1')\n", &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn use_default_false_no_default_checks() {
        let config = ProhibitedConfig {
            use_default: false,
            extra: vec![],
            remove: vec![],
        };
        let diags = check_with_config("x = eval('1+1')\n", &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn extra_and_default_combined() {
        let config = ProhibitedConfig {
            use_default: true,
            extra: vec!["print".into()],
            remove: vec![],
        };
        let diags = check_with_config("print('hello')\nx = eval('1+1')\n", &config);
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn parse_error_returns_empty() {
        let diags = check("def foo(:\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn diagnostic_contains_source_line() {
        let diags = check("x = eval('1+1')\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].source_line.as_deref(), Some("x = eval('1+1')"));
    }

    #[test]
    fn rule_id_has_readability_prefix() {
        let diags = check("x = eval('1+1')\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].rule_id.starts_with("readability-"));
    }

    #[test]
    fn multiple_prohibited_calls() {
        let diags = check("x = eval('1')\ny = exec('2')\n");
        assert_eq!(diags.len(), 2);
    }

    #[test]
    fn nested_prohibited_call() {
        let diags = check("x = foo(eval('1+1'))\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn prohibited_in_function_body() {
        let diags = check("def foo():\n    x = eval('1+1')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
    }

    #[test]
    fn prohibited_in_class_body() {
        let diags = check("class Foo:\n    x = eval('1')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
    }

    #[test]
    fn prohibited_in_decorator() {
        let diags = check("@eval\ndef foo():\n    pass\n");
        // A decorator is not a Call node in this case, so no diagnostic
        // But if it's a call: @eval('x')
        assert!(diags.is_empty());
    }

    #[test]
    fn prohibited_in_decorator_call() {
        let diags = check("@eval('x')\ndef foo():\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
    }

    #[test]
    fn attribute_chain() {
        let config = ProhibitedConfig {
            use_default: false,
            extra: vec!["a.b.c".into()],
            remove: vec![],
        };
        let diags = check_with_config("a.b.c()\n", &config);
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
    }

    #[test]
    fn detect_subprocess_call() {
        let diags = check("import subprocess\nsubprocess.call('ls')\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("subprocess.call"));
    }

    #[test]
    fn eval_in_default_arg() {
        let diags = check("def foo(x=eval('1')):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn eval_in_type_annotation() {
        let diags = check("def foo(x: eval('int')):\n    pass\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn eval_in_subscript_assign_target() {
        let diags = check("a[eval('x')] = 1\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn eval_in_aug_assign_target() {
        let diags = check("a[eval('x')] += 1\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn eval_in_comprehension_condition() {
        let diags = check("result = [x for x in items if eval('x')]\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("eval"));
    }

    #[test]
    fn os_system_as_value_not_flagged() {
        let diags = check("f = os.system\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn subprocess_call_with_shell_true() {
        let diags = check("import subprocess\nsubprocess.call('ls', shell=True)\n");
        assert_eq!(rule_ids(&diags), vec!["readability-prohibited"]);
        assert!(diags[0].message.contains("subprocess.call"));
    }

    #[test]
    fn subprocess_call_with_shell_false_not_flagged() {
        let diags = check("import subprocess\nsubprocess.call('ls', shell=False)\n");
        assert!(diags.is_empty());
    }
}
