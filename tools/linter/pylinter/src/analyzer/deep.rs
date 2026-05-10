use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::AnalysisConfig;
use rustpython_parser::ast::{self, Ranged};
use std::collections::{HashMap, HashSet};

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
    walk_unreachable_code(body, &lines, source, &mut locator, &mut diags);
    walk_unused_variables(body, &lines, source, &mut locator, &mut diags);
    walk_shadow_builtin(body, &lines, source, &mut locator, &mut diags);
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

// ── Unreachable code ────────────────────────────────────────────────────────

fn walk_unreachable_code(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                check_body_unreachable(&f.body, lines, source, locator, diags);
                walk_unreachable_code(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                check_body_unreachable(&f.body, lines, source, locator, diags);
                walk_unreachable_code(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_unreachable_code(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                check_body_unreachable(&i.body, lines, source, locator, diags);
                walk_unreachable_code(&i.body, lines, source, locator, diags);
                check_body_unreachable(&i.orelse, lines, source, locator, diags);
                walk_unreachable_code(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                check_body_unreachable(&f.body, lines, source, locator, diags);
                walk_unreachable_code(&f.body, lines, source, locator, diags);
                check_body_unreachable(&f.orelse, lines, source, locator, diags);
                walk_unreachable_code(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                check_body_unreachable(&f.body, lines, source, locator, diags);
                walk_unreachable_code(&f.body, lines, source, locator, diags);
                check_body_unreachable(&f.orelse, lines, source, locator, diags);
                walk_unreachable_code(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                check_body_unreachable(&w.body, lines, source, locator, diags);
                walk_unreachable_code(&w.body, lines, source, locator, diags);
                check_body_unreachable(&w.orelse, lines, source, locator, diags);
                walk_unreachable_code(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                check_body_unreachable(&w.body, lines, source, locator, diags);
                walk_unreachable_code(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                check_body_unreachable(&w.body, lines, source, locator, diags);
                walk_unreachable_code(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                check_body_unreachable(&t.body, lines, source, locator, diags);
                walk_unreachable_code(&t.body, lines, source, locator, diags);
                check_body_unreachable(&t.orelse, lines, source, locator, diags);
                walk_unreachable_code(&t.orelse, lines, source, locator, diags);
                check_body_unreachable(&t.finalbody, lines, source, locator, diags);
                walk_unreachable_code(&t.finalbody, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    check_body_unreachable(&h.body, lines, source, locator, diags);
                    walk_unreachable_code(&h.body, lines, source, locator, diags);
                }
            }
            ast::Stmt::TryStar(t) => {
                check_body_unreachable(&t.body, lines, source, locator, diags);
                walk_unreachable_code(&t.body, lines, source, locator, diags);
                check_body_unreachable(&t.orelse, lines, source, locator, diags);
                walk_unreachable_code(&t.orelse, lines, source, locator, diags);
                check_body_unreachable(&t.finalbody, lines, source, locator, diags);
                walk_unreachable_code(&t.finalbody, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    check_body_unreachable(&h.body, lines, source, locator, diags);
                    walk_unreachable_code(&h.body, lines, source, locator, diags);
                }
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    check_body_unreachable(&case.body, lines, source, locator, diags);
                    walk_unreachable_code(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn is_terminal_stmt(stmt: &ast::Stmt) -> bool {
    matches!(
        stmt,
        ast::Stmt::Return(_) | ast::Stmt::Break(_) | ast::Stmt::Continue(_) | ast::Stmt::Raise(_)
    )
}

fn check_body_unreachable(
    body: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    let mut after_terminal = false;
    for stmt in body {
        if after_terminal {
            if matches!(stmt, ast::Stmt::Pass(_)) {
                continue;
            }
            if is_docstring_expr(stmt) {
                continue;
            }
            let loc = locator.locate(stmt.start());
            report(
                loc,
                "deadcode-unreachable",
                "Unreachable code",
                lines,
                source,
                diags,
            );
        }
        if is_terminal_stmt(stmt) {
            after_terminal = true;
        }
    }
}

fn is_docstring_expr(stmt: &ast::Stmt) -> bool {
    if let ast::Stmt::Expr(e) = stmt {
        if let ast::Expr::Constant(c) = &*e.value {
            return matches!(c.value, ast::Constant::Str(_));
        }
    }
    false
}

// ── Unused variables ────────────────────────────────────────────────────────

fn walk_unused_variables(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                check_fn_scope(f.body.as_slice(), &f.args, lines, source, locator, diags);
                walk_unused_variables(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                check_fn_scope(f.body.as_slice(), &f.args, lines, source, locator, diags);
                walk_unused_variables(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_unused_variables(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                walk_unused_variables(&i.body, lines, source, locator, diags);
                walk_unused_variables(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                walk_unused_variables(&f.body, lines, source, locator, diags);
                walk_unused_variables(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                walk_unused_variables(&f.body, lines, source, locator, diags);
                walk_unused_variables(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_unused_variables(&w.body, lines, source, locator, diags);
                walk_unused_variables(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                walk_unused_variables(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                walk_unused_variables(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_unused_variables(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_unused_variables(&h.body, lines, source, locator, diags);
                }
                walk_unused_variables(&t.orelse, lines, source, locator, diags);
                walk_unused_variables(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_unused_variables(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    walk_unused_variables(&h.body, lines, source, locator, diags);
                }
                walk_unused_variables(&t.orelse, lines, source, locator, diags);
                walk_unused_variables(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    walk_unused_variables(&case.body, lines, source, locator, diags);
                }
            }
            _ => {}
        }
    }
}

fn check_fn_scope(
    body: &[ast::Stmt],
    args: &ast::Arguments,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    let mut assigned: HashMap<String, usize> = HashMap::new();
    let mut referenced: HashSet<String> = HashSet::new();
    let mut global_nonlocal: HashSet<String> = HashSet::new();

    collect_param_names(args, &mut assigned, locator);
    collect_stmt_assignments(body, &mut assigned, &mut global_nonlocal, locator);
    collect_stmt_refs(body, &mut referenced);

    for (name, line) in &assigned {
        if name.starts_with('_') {
            continue;
        }
        if global_nonlocal.contains(name) {
            continue;
        }
        if !referenced.contains(name) {
            let source_line = lines.get(line - 1).copied().unwrap_or("");
            diags.push(Diagnostic::new_with_source(
                source.display_path(),
                *line,
                1,
                Severity::Warning,
                "deadcode-unused-variable",
                &format!("Variable '{}' is assigned but never used", name),
                source_line,
            ));
        }
    }
}

fn collect_param_names(
    args: &ast::Arguments,
    assigned: &mut HashMap<String, usize>,
    locator: &mut rustpython_parser::source_code::RandomLocator,
) {
    let mut add_name = |name: &str, start: ast::TextSize| {
        let line = locator.locate(start).row.to_usize();
        assigned.entry(name.to_string()).or_insert(line);
    };
    for arg in &args.posonlyargs {
        add_name(arg.def.arg.as_str(), arg.def.start());
    }
    for arg in &args.args {
        add_name(arg.def.arg.as_str(), arg.def.start());
    }
    if let Some(arg) = &args.vararg {
        add_name(arg.arg.as_str(), arg.start());
    }
    for arg in &args.kwonlyargs {
        add_name(arg.def.arg.as_str(), arg.def.start());
    }
    if let Some(arg) = &args.kwarg {
        add_name(arg.arg.as_str(), arg.start());
    }
}

fn collect_stmt_assignments(
    stmts: &[ast::Stmt],
    assigned: &mut HashMap<String, usize>,
    global_nonlocal: &mut HashSet<String>,
    locator: &mut rustpython_parser::source_code::RandomLocator,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::Global(g) => {
                for name in &g.names {
                    global_nonlocal.insert(name.to_string());
                }
            }
            ast::Stmt::Nonlocal(n) => {
                for name in &n.names {
                    global_nonlocal.insert(name.to_string());
                }
            }
            ast::Stmt::FunctionDef(f) => {
                let line = locator.locate(f.start()).row.to_usize();
                assigned.entry(f.name.to_string()).or_insert(line);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                let line = locator.locate(f.start()).row.to_usize();
                assigned.entry(f.name.to_string()).or_insert(line);
            }
            ast::Stmt::ClassDef(c) => {
                let line = locator.locate(c.start()).row.to_usize();
                assigned.entry(c.name.to_string()).or_insert(line);
            }
            ast::Stmt::Import(i) => {
                for alias in &i.names {
                    if let Some(asname) = &alias.asname {
                        let line = locator.locate(alias.start()).row.to_usize();
                        assigned.entry(asname.to_string()).or_insert(line);
                    } else {
                        let line = locator.locate(alias.start()).row.to_usize();
                        let name = alias.name.split('.').next().unwrap_or(&alias.name);
                        assigned.entry(name.to_string()).or_insert(line);
                    }
                }
            }
            ast::Stmt::ImportFrom(i) => {
                for alias in &i.names {
                    if let Some(asname) = &alias.asname {
                        let line = locator.locate(alias.start()).row.to_usize();
                        assigned.entry(asname.to_string()).or_insert(line);
                    } else {
                        let line = locator.locate(alias.start()).row.to_usize();
                        assigned.entry(alias.name.to_string()).or_insert(line);
                    }
                }
            }
            ast::Stmt::Assign(a) => {
                for target in &a.targets {
                    collect_target_names(target, assigned, locator);
                }
            }
            ast::Stmt::AnnAssign(a) => {
                collect_target_names(&a.target, assigned, locator);
            }
            ast::Stmt::AugAssign(a) => {
                collect_target_names(&a.target, assigned, locator);
            }
            ast::Stmt::For(f) => {
                collect_target_names(&f.target, assigned, locator);
            }
            ast::Stmt::AsyncFor(f) => {
                collect_target_names(&f.target, assigned, locator);
            }
            ast::Stmt::With(w) => {
                for item in &w.items {
                    if let Some(opt_vars) = &item.optional_vars {
                        collect_target_names(opt_vars, assigned, locator);
                    }
                }
            }
            ast::Stmt::AsyncWith(w) => {
                for item in &w.items {
                    if let Some(opt_vars) = &item.optional_vars {
                        collect_target_names(opt_vars, assigned, locator);
                    }
                }
            }
            ast::Stmt::If(i) => {
                collect_stmt_assignments(&i.body, assigned, global_nonlocal, locator);
                collect_stmt_assignments(&i.orelse, assigned, global_nonlocal, locator);
            }
            ast::Stmt::While(w) => {
                collect_stmt_assignments(&w.body, assigned, global_nonlocal, locator);
                collect_stmt_assignments(&w.orelse, assigned, global_nonlocal, locator);
            }
            ast::Stmt::Try(t) => {
                collect_stmt_assignments(&t.body, assigned, global_nonlocal, locator);
                collect_stmt_assignments(&t.orelse, assigned, global_nonlocal, locator);
                collect_stmt_assignments(&t.finalbody, assigned, global_nonlocal, locator);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if let Some(name) = &h.name {
                        let line = locator.locate(h.start()).row.to_usize();
                        assigned.entry(name.to_string()).or_insert(line);
                    }
                    collect_stmt_assignments(&h.body, assigned, global_nonlocal, locator);
                }
            }
            ast::Stmt::TryStar(t) => {
                collect_stmt_assignments(&t.body, assigned, global_nonlocal, locator);
                collect_stmt_assignments(&t.orelse, assigned, global_nonlocal, locator);
                collect_stmt_assignments(&t.finalbody, assigned, global_nonlocal, locator);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if let Some(name) = &h.name {
                        let line = locator.locate(h.start()).row.to_usize();
                        assigned.entry(name.to_string()).or_insert(line);
                    }
                    collect_stmt_assignments(&h.body, assigned, global_nonlocal, locator);
                }
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    collect_pattern_names(&case.pattern, assigned, locator);
                    collect_stmt_assignments(&case.body, assigned, global_nonlocal, locator);
                }
            }
            _ => {}
        }
    }
}

fn collect_target_names(
    target: &ast::Expr,
    assigned: &mut HashMap<String, usize>,
    locator: &mut rustpython_parser::source_code::RandomLocator,
) {
    match target {
        ast::Expr::Name(n) => {
            let line = locator.locate(n.start()).row.to_usize();
            assigned.entry(n.id.to_string()).or_insert(line);
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                collect_target_names(elt, assigned, locator);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                collect_target_names(elt, assigned, locator);
            }
        }
        ast::Expr::Starred(s) => {
            collect_target_names(&s.value, assigned, locator);
        }
        _ => {}
    }
}

fn collect_pattern_names(
    pattern: &ast::Pattern,
    assigned: &mut HashMap<String, usize>,
    locator: &mut rustpython_parser::source_code::RandomLocator,
) {
    match pattern {
        ast::Pattern::MatchAs(m) => {
            if let Some(name) = &m.name {
                let line = locator.locate(m.start()).row.to_usize();
                assigned.entry(name.to_string()).or_insert(line);
            }
            if let Some(pat) = &m.pattern {
                collect_pattern_names(pat, assigned, locator);
            }
        }
        ast::Pattern::MatchOr(m) => {
            for pat in &m.patterns {
                collect_pattern_names(pat, assigned, locator);
            }
        }
        ast::Pattern::MatchSequence(m) => {
            for pat in &m.patterns {
                collect_pattern_names(pat, assigned, locator);
            }
        }
        ast::Pattern::MatchMapping(m) => {
            if let Some(rest) = &m.rest {
                let line = locator.locate(m.start()).row.to_usize();
                assigned.entry(rest.to_string()).or_insert(line);
            }
            for pat in &m.patterns {
                collect_pattern_names(pat, assigned, locator);
            }
        }
        ast::Pattern::MatchClass(m) => {
            for pat in &m.patterns {
                collect_pattern_names(pat, assigned, locator);
            }
            for pat in &m.kwd_patterns {
                collect_pattern_names(pat, assigned, locator);
            }
        }
        ast::Pattern::MatchStar(m) => {
            if let Some(name) = &m.name {
                let line = locator.locate(m.start()).row.to_usize();
                assigned.entry(name.to_string()).or_insert(line);
            }
        }
        _ => {}
    }
}

fn collect_stmt_refs(stmts: &[ast::Stmt], referenced: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(_) | ast::Stmt::AsyncFunctionDef(_) | ast::Stmt::ClassDef(_) => {
                continue;
            }
            ast::Stmt::Global(_) | ast::Stmt::Nonlocal(_) => continue,
            _ => {}
        }
        collect_stmt_refs_inner(stmt, referenced);
    }
}

fn collect_stmt_refs_inner(stmt: &ast::Stmt, referenced: &mut HashSet<String>) {
    match stmt {
        ast::Stmt::FunctionDef(f) => {
            collect_expr_refs(&f.decorator_list, referenced);
            collect_args_default_refs(&f.args, referenced);
        }
        ast::Stmt::AsyncFunctionDef(f) => {
            collect_expr_refs(&f.decorator_list, referenced);
            collect_args_default_refs(&f.args, referenced);
        }
        ast::Stmt::ClassDef(c) => {
            collect_expr_refs(&c.decorator_list, referenced);
            for base in &c.bases {
                collect_expr_name_refs(base, referenced);
            }
            for kw in &c.keywords {
                collect_expr_name_refs(&kw.value, referenced);
            }
        }
        ast::Stmt::Return(r) => {
            if let Some(v) = &r.value {
                collect_expr_name_refs(v, referenced);
            }
        }
        ast::Stmt::Assign(a) => {
            collect_expr_name_refs(&a.value, referenced);
            for t in &a.targets {
                collect_expr_refs_in_target(t, referenced);
            }
        }
        ast::Stmt::AnnAssign(a) => {
            collect_expr_name_refs(&a.annotation, referenced);
            if let Some(v) = &a.value {
                collect_expr_name_refs(v, referenced);
            }
        }
        ast::Stmt::AugAssign(a) => {
            collect_target_refs(&a.target, referenced);
            collect_expr_name_refs(&a.value, referenced);
        }
        ast::Stmt::For(f) => {
            collect_expr_name_refs(&f.iter, referenced);
            for s in &f.body {
                collect_stmt_refs_inner(s, referenced);
            }
            for s in &f.orelse {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::AsyncFor(f) => {
            collect_expr_name_refs(&f.iter, referenced);
            for s in &f.body {
                collect_stmt_refs_inner(s, referenced);
            }
            for s in &f.orelse {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::While(w) => {
            collect_expr_name_refs(&w.test, referenced);
            for s in &w.body {
                collect_stmt_refs_inner(s, referenced);
            }
            for s in &w.orelse {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::If(i) => {
            collect_expr_name_refs(&i.test, referenced);
            for s in &i.body {
                collect_stmt_refs_inner(s, referenced);
            }
            for s in &i.orelse {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::With(w) => {
            for item in &w.items {
                collect_expr_name_refs(&item.context_expr, referenced);
            }
            for s in &w.body {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::AsyncWith(w) => {
            for item in &w.items {
                collect_expr_name_refs(&item.context_expr, referenced);
            }
            for s in &w.body {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::Raise(r) => {
            if let Some(exc) = &r.exc {
                collect_expr_name_refs(exc, referenced);
            }
            if let Some(cause) = &r.cause {
                collect_expr_name_refs(cause, referenced);
            }
        }
        ast::Stmt::Try(t) => {
            for s in &t.body {
                collect_stmt_refs_inner(s, referenced);
            }
            for h in &t.handlers {
                let ast::ExceptHandler::ExceptHandler(h) = h;
                if let Some(ty) = &h.type_ {
                    collect_expr_name_refs(ty, referenced);
                }
                for s in &h.body {
                    collect_stmt_refs_inner(s, referenced);
                }
            }
            for s in &t.orelse {
                collect_stmt_refs_inner(s, referenced);
            }
            for s in &t.finalbody {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::TryStar(t) => {
            for s in &t.body {
                collect_stmt_refs_inner(s, referenced);
            }
            for h in &t.handlers {
                let ast::ExceptHandler::ExceptHandler(h) = h;
                if let Some(ty) = &h.type_ {
                    collect_expr_name_refs(ty, referenced);
                }
                for s in &h.body {
                    collect_stmt_refs_inner(s, referenced);
                }
            }
            for s in &t.orelse {
                collect_stmt_refs_inner(s, referenced);
            }
            for s in &t.finalbody {
                collect_stmt_refs_inner(s, referenced);
            }
        }
        ast::Stmt::Assert(a) => {
            collect_expr_name_refs(&a.test, referenced);
            if let Some(msg) = &a.msg {
                collect_expr_name_refs(msg, referenced);
            }
        }
        ast::Stmt::Import(_) | ast::Stmt::ImportFrom(_) => {}
        ast::Stmt::Expr(e) => {
            collect_expr_name_refs(&e.value, referenced);
        }
        ast::Stmt::Match(m) => {
            collect_expr_name_refs(&m.subject, referenced);
            for case in &m.cases {
                if let Some(g) = &case.guard {
                    collect_expr_name_refs(g, referenced);
                }
                for s in &case.body {
                    collect_stmt_refs_inner(s, referenced);
                }
            }
        }
        ast::Stmt::Delete(d) => {
            for target in &d.targets {
                collect_expr_name_refs(target, referenced);
            }
        }
        _ => {}
    }
}

fn collect_args_default_refs(args: &ast::Arguments, referenced: &mut HashSet<String>) {
    for arg in args.posonlyargs.iter().chain(args.args.iter()).chain(args.kwonlyargs.iter()) {
        if let Some(default) = &arg.default {
            collect_expr_name_refs(default, referenced);
        }
    }
}

fn collect_expr_refs(exprs: &[ast::Expr], referenced: &mut HashSet<String>) {
    for expr in exprs {
        collect_expr_name_refs(expr, referenced);
    }
}

fn collect_generator_refs(generators: &[ast::Comprehension], referenced: &mut HashSet<String>) {
    for gen in generators {
        collect_expr_name_refs(&gen.iter, referenced);
        for if_ in &gen.ifs {
            collect_expr_name_refs(if_, referenced);
        }
    }
}

fn collect_expr_name_refs(expr: &ast::Expr, referenced: &mut HashSet<String>) {
    match expr {
        ast::Expr::Name(n) => {
            referenced.insert(n.id.to_string());
        }
        ast::Expr::BoolOp(b) => {
            for v in &b.values {
                collect_expr_name_refs(v, referenced);
            }
        }
        ast::Expr::BinOp(b) => {
            collect_expr_name_refs(&b.left, referenced);
            collect_expr_name_refs(&b.right, referenced);
        }
        ast::Expr::UnaryOp(u) => {
            collect_expr_name_refs(&u.operand, referenced);
        }
        ast::Expr::IfExp(i) => {
            collect_expr_name_refs(&i.test, referenced);
            collect_expr_name_refs(&i.body, referenced);
            collect_expr_name_refs(&i.orelse, referenced);
        }
        ast::Expr::Call(c) => {
            collect_expr_name_refs(&c.func, referenced);
            for arg in &c.args {
                collect_expr_name_refs(arg, referenced);
            }
            for kw in &c.keywords {
                collect_expr_name_refs(&kw.value, referenced);
            }
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                collect_expr_name_refs(elt, referenced);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                collect_expr_name_refs(elt, referenced);
            }
        }
        ast::Expr::Set(s) => {
            for elt in &s.elts {
                collect_expr_name_refs(elt, referenced);
            }
        }
        ast::Expr::Dict(d) => {
            for k in d.keys.iter().flatten() {
                collect_expr_name_refs(k, referenced);
            }
            for v in &d.values {
                collect_expr_name_refs(v, referenced);
            }
        }
        ast::Expr::Compare(cmp) => {
            collect_expr_name_refs(&cmp.left, referenced);
            for c in &cmp.comparators {
                collect_expr_name_refs(c, referenced);
            }
        }
        ast::Expr::Subscript(s) => {
            collect_expr_name_refs(&s.value, referenced);
            collect_expr_name_refs(&s.slice, referenced);
        }
        ast::Expr::Starred(s) => {
            collect_expr_name_refs(&s.value, referenced);
        }
        ast::Expr::Attribute(a) => {
            collect_expr_name_refs(&a.value, referenced);
        }
        ast::Expr::JoinedStr(js) => {
            for v in &js.values {
                collect_expr_name_refs(v, referenced);
            }
        }
        ast::Expr::FormattedValue(f) => {
            collect_expr_name_refs(&f.value, referenced);
        }
        ast::Expr::Slice(s) => {
            if let Some(l) = &s.lower {
                collect_expr_name_refs(l, referenced);
            }
            if let Some(u) = &s.upper {
                collect_expr_name_refs(u, referenced);
            }
            if let Some(st) = &s.step {
                collect_expr_name_refs(st, referenced);
            }
        }
        ast::Expr::ListComp(l) => {
            collect_expr_name_refs(&l.elt, referenced);
            collect_generator_refs(&l.generators, referenced);
        }
        ast::Expr::SetComp(s) => {
            collect_expr_name_refs(&s.elt, referenced);
            collect_generator_refs(&s.generators, referenced);
        }
        ast::Expr::DictComp(d) => {
            collect_expr_name_refs(&d.key, referenced);
            collect_expr_name_refs(&d.value, referenced);
            collect_generator_refs(&d.generators, referenced);
        }
        ast::Expr::GeneratorExp(g) => {
            collect_expr_name_refs(&g.elt, referenced);
            collect_generator_refs(&g.generators, referenced);
        }
        ast::Expr::Await(a) => {
            collect_expr_name_refs(&a.value, referenced);
        }
        ast::Expr::Yield(y) => {
            if let Some(v) = &y.value {
                collect_expr_name_refs(v, referenced);
            }
        }
        ast::Expr::YieldFrom(y) => {
            collect_expr_name_refs(&y.value, referenced);
        }
        _ => {}
    }
}

fn collect_target_refs(target: &ast::Expr, referenced: &mut HashSet<String>) {
    if let ast::Expr::Name(n) = target {
        referenced.insert(n.id.to_string());
    }
}

fn collect_expr_refs_in_target(target: &ast::Expr, referenced: &mut HashSet<String>) {
    match target {
        ast::Expr::Name(_) => {}
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                collect_expr_refs_in_target(elt, referenced);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                collect_expr_refs_in_target(elt, referenced);
            }
        }
        ast::Expr::Starred(s) => {
            collect_expr_name_refs(&s.value, referenced);
        }
        ast::Expr::Subscript(s) => {
            collect_expr_name_refs(&s.value, referenced);
            collect_expr_name_refs(&s.slice, referenced);
        }
        ast::Expr::Attribute(a) => {
            collect_expr_name_refs(&a.value, referenced);
        }
        _ => {}
    }
}

// ── Shadow builtins ─────────────────────────────────────────────────────────

const BUILTINS: &[&str] = &[
    "list", "dict", "set", "tuple", "str", "int", "float", "bool",
    "type", "id", "input", "print", "len", "range", "enumerate",
    "zip", "map", "filter", "sorted", "reversed", "sum", "min", "max",
    "abs", "round", "any", "all", "open", "hash", "dir", "vars",
    "super", "property", "staticmethod", "classmethod",
    "object", "Exception", "BaseException",
];

fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}

fn walk_shadow_builtin(
    stmts: &[ast::Stmt],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                check_fn_params_shadow(&f.args, lines, source, locator, diags);
                walk_shadow_builtin(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFunctionDef(f) => {
                check_fn_params_shadow(&f.args, lines, source, locator, diags);
                walk_shadow_builtin(&f.body, lines, source, locator, diags);
            }
            ast::Stmt::ClassDef(c) => {
                walk_shadow_builtin(&c.body, lines, source, locator, diags);
            }
            ast::Stmt::If(i) => {
                walk_shadow_builtin(&i.body, lines, source, locator, diags);
                walk_shadow_builtin(&i.orelse, lines, source, locator, diags);
            }
            ast::Stmt::For(f) => {
                check_target_shadow(&f.target, lines, source, locator, diags);
                walk_shadow_builtin(&f.body, lines, source, locator, diags);
                walk_shadow_builtin(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::AsyncFor(f) => {
                check_target_shadow(&f.target, lines, source, locator, diags);
                walk_shadow_builtin(&f.body, lines, source, locator, diags);
                walk_shadow_builtin(&f.orelse, lines, source, locator, diags);
            }
            ast::Stmt::While(w) => {
                walk_shadow_builtin(&w.body, lines, source, locator, diags);
                walk_shadow_builtin(&w.orelse, lines, source, locator, diags);
            }
            ast::Stmt::With(w) => {
                for item in &w.items {
                    if let Some(opt) = &item.optional_vars {
                        check_target_shadow(opt, lines, source, locator, diags);
                    }
                }
                walk_shadow_builtin(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::AsyncWith(w) => {
                for item in &w.items {
                    if let Some(opt) = &item.optional_vars {
                        check_target_shadow(opt, lines, source, locator, diags);
                    }
                }
                walk_shadow_builtin(&w.body, lines, source, locator, diags);
            }
            ast::Stmt::Try(t) => {
                walk_shadow_builtin(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if let Some(name) = &h.name {
                        if is_builtin(name) {
                            let loc = locator.locate(h.start());
                            report(
                                loc,
                                "bugprone-shadow-builtin",
                                &format!("Variable '{}' shadows a builtin", name),
                                lines,
                                source,
                                diags,
                            );
                        }
                    }
                    walk_shadow_builtin(&h.body, lines, source, locator, diags);
                }
                walk_shadow_builtin(&t.orelse, lines, source, locator, diags);
                walk_shadow_builtin(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::TryStar(t) => {
                walk_shadow_builtin(&t.body, lines, source, locator, diags);
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if let Some(name) = &h.name {
                        if is_builtin(name) {
                            let loc = locator.locate(h.start());
                            report(
                                loc,
                                "bugprone-shadow-builtin",
                                &format!("Variable '{}' shadows a builtin", name),
                                lines,
                                source,
                                diags,
                            );
                        }
                    }
                    walk_shadow_builtin(&h.body, lines, source, locator, diags);
                }
                walk_shadow_builtin(&t.orelse, lines, source, locator, diags);
                walk_shadow_builtin(&t.finalbody, lines, source, locator, diags);
            }
            ast::Stmt::Match(m) => {
                for case in &m.cases {
                    check_pattern_shadow(&case.pattern, lines, source, locator, diags);
                    walk_shadow_builtin(&case.body, lines, source, locator, diags);
                }
            }
            ast::Stmt::Assign(a) => {
                for target in &a.targets {
                    check_target_shadow(target, lines, source, locator, diags);
                }
            }
            ast::Stmt::AnnAssign(a) => {
                check_target_shadow(&a.target, lines, source, locator, diags);
            }
            ast::Stmt::AugAssign(a) => {
                check_target_shadow(&a.target, lines, source, locator, diags);
            }
            ast::Stmt::Import(i) => {
                check_import_aliases_shadow(&i.names, lines, source, locator, diags);
            }
            ast::Stmt::ImportFrom(i) => {
                check_import_aliases_shadow(&i.names, lines, source, locator, diags);
            }
            _ => {}
        }
    }
}

fn check_fn_params_shadow(
    args: &ast::Arguments,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    let mut check_name = |name: &str, start: ast::TextSize, diags: &mut Vec<Diagnostic>| {
        if is_builtin(name) {
            let loc = locator.locate(start);
            report(
                loc,
                "bugprone-shadow-builtin",
                &format!("Parameter '{}' shadows a builtin", name),
                lines,
                source,
                diags,
            );
        }
    };
    for arg in &args.posonlyargs {
        check_name(arg.def.arg.as_str(), arg.def.start(), diags);
    }
    for arg in &args.args {
        check_name(arg.def.arg.as_str(), arg.def.start(), diags);
    }
    if let Some(arg) = &args.vararg {
        check_name(arg.arg.as_str(), arg.start(), diags);
    }
    for arg in &args.kwonlyargs {
        check_name(arg.def.arg.as_str(), arg.def.start(), diags);
    }
    if let Some(arg) = &args.kwarg {
        check_name(arg.arg.as_str(), arg.start(), diags);
    }
}

fn check_import_aliases_shadow(
    names: &[ast::Alias],
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    for alias in names {
        if let Some(asname) = &alias.asname {
            if is_builtin(asname) {
                let loc = locator.locate(alias.start());
                report(
                    loc,
                    "bugprone-shadow-builtin",
                    &format!("Import alias '{}' shadows a builtin", asname),
                    lines,
                    source,
                    diags,
                );
            }
        }
    }
}

fn check_target_shadow(
    target: &ast::Expr,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    match target {
        ast::Expr::Name(n) => {
            if is_builtin(n.id.as_str()) {
                let loc = locator.locate(n.start());
                report(
                    loc,
                    "bugprone-shadow-builtin",
                    &format!("Variable '{}' shadows a builtin", n.id),
                    lines,
                    source,
                    diags,
                );
            }
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                check_target_shadow(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                check_target_shadow(elt, lines, source, locator, diags);
            }
        }
        ast::Expr::Starred(s) => {
            check_target_shadow(&s.value, lines, source, locator, diags);
        }
        _ => {}
    }
}

fn check_pattern_shadow(
    pattern: &ast::Pattern,
    lines: &[&str],
    source: &SourceFile,
    locator: &mut rustpython_parser::source_code::RandomLocator,
    diags: &mut Vec<Diagnostic>,
) {
    match pattern {
        ast::Pattern::MatchAs(m) => {
            if let Some(name) = &m.name {
                if is_builtin(name) {
                    let loc = locator.locate(m.start());
                    report(
                        loc,
                        "bugprone-shadow-builtin",
                        &format!("Variable '{}' shadows a builtin", name),
                        lines,
                        source,
                        diags,
                    );
                }
            }
            if let Some(p) = &m.pattern {
                check_pattern_shadow(p, lines, source, locator, diags);
            }
        }
        ast::Pattern::MatchOr(m) => {
            for p in &m.patterns {
                check_pattern_shadow(p, lines, source, locator, diags);
            }
        }
        ast::Pattern::MatchSequence(m) => {
            for p in &m.patterns {
                check_pattern_shadow(p, lines, source, locator, diags);
            }
        }
        ast::Pattern::MatchMapping(m) => {
            if let Some(rest) = &m.rest {
                if is_builtin(rest) {
                    let loc = locator.locate(m.start());
                    report(
                        loc,
                        "bugprone-shadow-builtin",
                        &format!("Variable '{}' shadows a builtin", rest),
                        lines,
                        source,
                        diags,
                    );
                }
            }
            for p in &m.patterns {
                check_pattern_shadow(p, lines, source, locator, diags);
            }
        }
        ast::Pattern::MatchClass(m) => {
            for p in &m.patterns {
                check_pattern_shadow(p, lines, source, locator, diags);
            }
            for p in &m.kwd_patterns {
                check_pattern_shadow(p, lines, source, locator, diags);
            }
        }
        ast::Pattern::MatchStar(m) => {
            if let Some(name) = &m.name {
                if is_builtin(name) {
                    let loc = locator.locate(m.start());
                    report(
                        loc,
                        "bugprone-shadow-builtin",
                        &format!("Variable '{}' shadows a builtin", name),
                        lines,
                        source,
                        diags,
                    );
                }
            }
        }
        _ => {}
    }
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

    // ── Unreachable code ──────────────────────────────────────────────────

    #[test]
    fn unreachable_after_return() {
        let src = "def foo():\n    return 1\n    x = 2\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn reachable_code_ok() {
        let src = "def foo():\n    x = 1\n    return x\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn unreachable_after_break() {
        let src = "def foo():\n    for x in y:\n        break\n        z = 1\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn unreachable_after_continue() {
        let src = "def foo():\n    for x in y:\n        continue\n        z = 1\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn unreachable_after_raise() {
        let src = "def foo():\n    raise ValueError\n    x = 1\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn unreachable_pass_ok() {
        let src = "def foo():\n    return 1\n    pass\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn unreachable_docstring_ok() {
        let src = "def foo():\n    return 1\n    \"\"\"unreachable doc.\"\"\"\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    #[test]
    fn unreachable_multiple_stmts() {
        let src = "def foo():\n    return 1\n    x = 2\n    y = 3\n";
        let diags = analyze(src);
        let unreachable = rule_ids(&diags)
            .iter()
            .filter(|&&r| r == "deadcode-unreachable")
            .count();
        assert_eq!(unreachable, 2);
    }

    #[test]
    fn unreachable_in_class_ok() {
        let src = "class Foo:\n    return 1\n    x = 2\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unreachable"));
    }

    // ── Unused variables ──────────────────────────────────────────────────

    #[test]
    fn unused_variable() {
        let src = "def foo():\n    x = 1\n    return 2\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn used_variable_ok() {
        let src = "def foo():\n    x = 1\n    return x\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn underscore_prefix_ok() {
        let src = "def foo():\n    _unused = 1\n    return 2\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn unused_param() {
        let src = "def foo(x):\n    return 1\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn used_param_ok() {
        let src = "def foo(x):\n    return x\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn underscore_param_ok() {
        let src = "def foo(_x):\n    return 1\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn global_var_not_flagged() {
        let src = "def foo():\n    global x\n    x = 1\n    return 2\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn nonlocal_var_not_flagged() {
        let src = "def foo():\n    x = 1\n    def bar():\n        nonlocal x\n        x = 2\n        return x\n    return bar\n";
        let diags = analyze(src);
        // x is unused in foo()'s scope — bar() is a separate scope
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn unused_for_target() {
        let src = "def foo():\n    for item in [1, 2, 3]:\n        pass\n    return 1\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn used_for_target_ok() {
        let src = "def foo():\n    for item in [1, 2, 3]:\n        return item\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn nested_fn_creates_own_scope() {
        let src = "def foo():\n    x = 1\n    def bar():\n        return x\n    return bar\n";
        let diags = analyze(src);
        // x is unused in foo()'s scope — bar() is a separate scope
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn tuple_unpack_unused() {
        let src = "def foo():\n    a, b = 1, 2\n    return a\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    #[test]
    fn augmented_assign_read() {
        let src = "def foo():\n    x = 1\n    x += 2\n    return x\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"deadcode-unused-variable"));
    }

    // ── Shadow builtins ──────────────────────────────────────────────────

    #[test]
    fn shadow_builtin_list() {
        let src = "def foo(list):\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn normal_name_ok() {
        let src = "def foo(items):\n    pass\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn shadow_builtin_assign() {
        let src = "def foo():\n    list = [1, 2, 3]\n    return list\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn shadow_builtin_for_target() {
        let src = "def foo():\n    for set in [1, 2, 3]:\n        pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn shadow_builtin_except_name() {
        let src = "def foo():\n    try:\n        pass\n    except Exception as Exception:\n        pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn no_shadow_non_builtin() {
        let src = "def foo(items):\n    items = [1]\n    return items\n";
        let diags = analyze(src);
        assert!(!rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn shadow_builtin_print() {
        let src = "def foo(print):\n    pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn shadow_builtin_in_with() {
        let src = "def foo():\n    with open('f') as input:\n        pass\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
    }

    #[test]
    fn shadow_builtin_import_alias() {
        let src = "def foo():\n    import os as str\n    return str\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
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
    fn multiple_deep_issues() {
        let src = "def foo(list):\n    x = 1\n    return 2\n    y = 3\n";
        let diags = analyze(src);
        assert!(rule_ids(&diags).contains(&"bugprone-shadow-builtin"));
        assert!(rule_ids(&diags).contains(&"deadcode-unused-variable"));
        assert!(rule_ids(&diags).contains(&"deadcode-unreachable"));
    }
}
