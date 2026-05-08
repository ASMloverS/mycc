use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;
use crate::config::UnusedImportConfig;
use rustpython_parser::ast::{self, Ranged};
use std::collections::HashSet;

pub fn check_unused_import(source: &SourceFile, config: &UnusedImportConfig) -> Vec<Diagnostic> {
    if !config.enabled {
        return vec![];
    }

    if source.path.file_name().map_or(false, |n| n == "__init__.py") {
        return vec![];
    }

    let mut diags = Vec::new();

    let Ok(ast) =
        rustpython_parser::parse(&source.content, rustpython_parser::Mode::Module, "<input>")
    else {
        return diags;
    };

    let lines = source.lines();
    let mut locator = rustpython_parser::source_code::RandomLocator::new(&source.content);

    let body = match &ast {
        ast::Mod::Module(m) => &m.body,
        _ => return diags,
    };

    let mut imports: Vec<ImportInfo> = Vec::new();
    let mut references: HashSet<String> = HashSet::new();

    collect_imports_and_refs(body, &mut imports, &mut references, false);

    for imp in &imports {
        if !references.contains(&imp.name) {
            let line = locator.locate(imp.position).row.to_usize();
            let source_line = lines.get(line - 1).copied().unwrap_or("");
            diags.push(Diagnostic::new_with_source(
                source.display_path(),
                line,
                1,
                Severity::Warning,
                "readability-unused-import",
                &format!("Unused import: {}", imp.name),
                source_line,
            ));
        }
    }

    diags
}

struct ImportInfo {
    name: String,
    position: ast::TextSize,
}

fn alias_effective_name(alias: &ast::Alias) -> Option<String> {
    if alias.name.as_str() == "*" {
        return None;
    }
    Some(
        alias
            .asname
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| alias.name.split('.').next().unwrap_or(&alias.name).to_string()),
    )
}

fn collect_imports_and_refs(
    stmts: &[ast::Stmt],
    imports: &mut Vec<ImportInfo>,
    references: &mut HashSet<String>,
    inside_type_checking: bool,
) {
    for stmt in stmts.iter() {
        match stmt {
            ast::Stmt::Import(imp) => {
                if inside_type_checking {
                    continue;
                }
                for alias in &imp.names {
                    if let Some(name) = alias_effective_name(alias) {
                        imports.push(ImportInfo {
                            name,
                            position: alias.start(),
                        });
                    }
                }
            }
            ast::Stmt::ImportFrom(imp) => {
                if inside_type_checking {
                    continue;
                }
                for alias in &imp.names {
                    if let Some(name) = alias_effective_name(alias) {
                        imports.push(ImportInfo {
                            name,
                            position: alias.start(),
                        });
                    }
                }
            }
            ast::Stmt::If(if_stmt) => {
                let is_type_checking = is_type_checking_test(&if_stmt.test);
                collect_expr_refs(&if_stmt.test, references);
                collect_imports_and_refs(
                    &if_stmt.body,
                    imports,
                    references,
                    inside_type_checking || is_type_checking,
                );
                collect_imports_and_refs(
                    &if_stmt.orelse,
                    imports,
                    references,
                    inside_type_checking,
                );
            }
            _ => {
                collect_stmt_refs(stmt, references);
            }
        }
    }
}

fn is_type_checking_test(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Name(n) => n.id.as_str() == "TYPE_CHECKING",
        ast::Expr::Attribute(a) => {
            if let ast::Expr::Name(n) = a.value.as_ref() {
                n.id.as_str() == "typing" && a.attr.as_str() == "TYPE_CHECKING"
            } else {
                false
            }
        }
        _ => false,
    }
}

fn collect_args_refs(args: &ast::Arguments, refs: &mut HashSet<String>) {
    for arg in args.posonlyargs.iter().chain(&args.args).chain(&args.kwonlyargs) {
        if let Some(ref ann) = arg.def.annotation {
            collect_expr_refs(ann, refs);
        }
        if let Some(ref default) = arg.default {
            collect_expr_refs(default, refs);
        }
    }
    if let Some(ref vararg) = args.vararg {
        if let Some(ref ann) = vararg.annotation {
            collect_expr_refs(ann, refs);
        }
    }
    if let Some(ref kwarg) = args.kwarg {
        if let Some(ref ann) = kwarg.annotation {
            collect_expr_refs(ann, refs);
        }
    }
}

fn collect_target_refs(target: &ast::Expr, refs: &mut HashSet<String>) {
    match target {
        ast::Expr::Name(n) => {
            if matches!(n.ctx, ast::ExprContext::Load) {
                refs.insert(n.id.to_string());
            }
        }
        ast::Expr::Attribute(a) => {
            collect_expr_refs(&a.value, refs);
        }
        ast::Expr::Subscript(s) => {
            collect_expr_refs(&s.value, refs);
            collect_expr_refs(&s.slice, refs);
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                collect_target_refs(elt, refs);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                collect_target_refs(elt, refs);
            }
        }
        ast::Expr::Starred(s) => {
            collect_target_refs(&s.value, refs);
        }
        _ => {}
    }
}

fn collect_stmt_refs(stmt: &ast::Stmt, refs: &mut HashSet<String>) {
    match stmt {
        ast::Stmt::FunctionDef(f) => collect_func_refs(
            &f.decorator_list, &f.args, &f.returns, &f.body, refs,
        ),
        ast::Stmt::AsyncFunctionDef(f) => collect_func_refs(
            &f.decorator_list, &f.args, &f.returns, &f.body, refs,
        ),
        ast::Stmt::ClassDef(c) => {
            for base in &c.bases {
                collect_expr_refs(base, refs);
            }
            for kw in &c.keywords {
                collect_expr_refs(&kw.value, refs);
            }
            for deco in &c.decorator_list {
                collect_expr_refs(deco, refs);
            }
            collect_body_refs(&c.body, refs);
        }
        ast::Stmt::Return(r) => {
            if let Some(ref value) = r.value {
                collect_expr_refs(value, refs);
            }
        }
        ast::Stmt::Assign(a) => {
            for target in &a.targets {
                collect_target_refs(target, refs);
            }
            collect_expr_refs(&a.value, refs);
        }
        ast::Stmt::AugAssign(a) => {
            collect_target_refs(&a.target, refs);
            collect_expr_refs(&a.value, refs);
        }
        ast::Stmt::AnnAssign(a) => {
            collect_target_refs(&a.target, refs);
            collect_expr_refs(&a.annotation, refs);
            if let Some(ref value) = a.value {
                collect_expr_refs(value, refs);
            }
        }
        ast::Stmt::For(f) => collect_for_refs(&f.iter, &f.body, &f.orelse, refs),
        ast::Stmt::AsyncFor(f) => collect_for_refs(&f.iter, &f.body, &f.orelse, refs),
        ast::Stmt::While(w) => {
            collect_expr_refs(&w.test, refs);
            collect_body_refs(&w.body, refs);
            collect_body_refs(&w.orelse, refs);
        }
        ast::Stmt::With(w) => collect_with_refs(&w.items, &w.body, refs),
        ast::Stmt::AsyncWith(w) => collect_with_refs(&w.items, &w.body, refs),
        ast::Stmt::Raise(r) => {
            if let Some(ref exc) = r.exc {
                collect_expr_refs(exc, refs);
            }
            if let Some(ref cause) = r.cause {
                collect_expr_refs(cause, refs);
            }
        }
        ast::Stmt::Assert(a) => {
            collect_expr_refs(&a.test, refs);
            if let Some(ref msg) = a.msg {
                collect_expr_refs(msg, refs);
            }
        }
        ast::Stmt::Try(t) => collect_try_refs(&t.body, &t.handlers, &t.orelse, &t.finalbody, refs),
        ast::Stmt::TryStar(t) => collect_try_refs(&t.body, &t.handlers, &t.orelse, &t.finalbody, refs),
        ast::Stmt::Match(m) => {
            collect_expr_refs(&m.subject, refs);
            for case in &m.cases {
                if let Some(ref guard) = case.guard {
                    collect_expr_refs(guard, refs);
                }
                collect_body_refs(&case.body, refs);
            }
        }
        ast::Stmt::Expr(e) => {
            collect_expr_refs(&e.value, refs);
        }
        ast::Stmt::Delete(d) => {
            for target in &d.targets {
                collect_expr_refs(target, refs);
            }
        }
        ast::Stmt::TypeAlias(t) => {
            collect_expr_refs(&t.value, refs);
        }
        _ => {}
    }
}

fn collect_func_refs(
    decorator_list: &[ast::Expr],
    args: &ast::Arguments,
    returns: &Option<Box<ast::Expr>>,
    body: &[ast::Stmt],
    refs: &mut HashSet<String>,
) {
    for deco in decorator_list {
        collect_expr_refs(deco, refs);
    }
    collect_args_refs(args, refs);
    if let Some(ref ret) = returns {
        collect_expr_refs(ret, refs);
    }
    collect_body_refs(body, refs);
}

fn collect_for_refs(
    iter: &ast::Expr,
    body: &[ast::Stmt],
    orelse: &[ast::Stmt],
    refs: &mut HashSet<String>,
) {
    collect_expr_refs(iter, refs);
    collect_body_refs(body, refs);
    collect_body_refs(orelse, refs);
}

fn collect_with_refs(
    items: &[ast::WithItem],
    body: &[ast::Stmt],
    refs: &mut HashSet<String>,
) {
    for item in items {
        collect_expr_refs(&item.context_expr, refs);
    }
    collect_body_refs(body, refs);
}

fn collect_try_refs(
    body: &[ast::Stmt],
    handlers: &[ast::ExceptHandler],
    orelse: &[ast::Stmt],
    finalbody: &[ast::Stmt],
    refs: &mut HashSet<String>,
) {
    collect_body_refs(body, refs);
    for handler in handlers {
        let ast::ExceptHandler::ExceptHandler(h) = handler;
        if let Some(ref exc) = h.type_ {
            collect_expr_refs(exc, refs);
        }
        collect_body_refs(&h.body, refs);
    }
    collect_body_refs(orelse, refs);
    collect_body_refs(finalbody, refs);
}

fn collect_body_refs(stmts: &[ast::Stmt], refs: &mut HashSet<String>) {
    for stmt in stmts {
        collect_stmt_refs(stmt, refs);
    }
}

fn collect_expr_refs(expr: &ast::Expr, refs: &mut HashSet<String>) {
    match expr {
        ast::Expr::Name(n) => {
            refs.insert(n.id.to_string());
        }
        ast::Expr::Attribute(a) => {
            if let ast::Expr::Name(n) = a.value.as_ref() {
                refs.insert(n.id.to_string());
            } else {
                collect_expr_refs(&a.value, refs);
            }
        }
        ast::Expr::Call(c) => {
            collect_expr_refs(&c.func, refs);
            for arg in &c.args {
                collect_expr_refs(arg, refs);
            }
            for kw in &c.keywords {
                collect_expr_refs(&kw.value, refs);
            }
        }
        ast::Expr::BinOp(b) => {
            collect_expr_refs(&b.left, refs);
            collect_expr_refs(&b.right, refs);
        }
        ast::Expr::UnaryOp(u) => {
            collect_expr_refs(&u.operand, refs);
        }
        ast::Expr::BoolOp(b) => {
            for v in &b.values {
                collect_expr_refs(v, refs);
            }
        }
        ast::Expr::Compare(c) => {
            collect_expr_refs(&c.left, refs);
            for comp in &c.comparators {
                collect_expr_refs(comp, refs);
            }
        }
        ast::Expr::Subscript(s) => {
            collect_expr_refs(&s.value, refs);
            collect_expr_refs(&s.slice, refs);
        }
        ast::Expr::Slice(s) => {
            if let Some(ref lower) = s.lower {
                collect_expr_refs(lower, refs);
            }
            if let Some(ref upper) = s.upper {
                collect_expr_refs(upper, refs);
            }
            if let Some(ref step) = s.step {
                collect_expr_refs(step, refs);
            }
        }
        ast::Expr::Tuple(t) => {
            for elt in &t.elts {
                collect_expr_refs(elt, refs);
            }
        }
        ast::Expr::List(l) => {
            for elt in &l.elts {
                collect_expr_refs(elt, refs);
            }
        }
        ast::Expr::Set(s) => {
            for elt in &s.elts {
                collect_expr_refs(elt, refs);
            }
        }
        ast::Expr::Dict(d) => {
            for (k, v) in d.keys.iter().zip(&d.values) {
                if let Some(key) = k {
                    collect_expr_refs(key, refs);
                }
                collect_expr_refs(v, refs);
            }
        }
        ast::Expr::IfExp(i) => {
            collect_expr_refs(&i.test, refs);
            collect_expr_refs(&i.body, refs);
            collect_expr_refs(&i.orelse, refs);
        }
        ast::Expr::Lambda(l) => {
            collect_args_refs(&l.args, refs);
            collect_expr_refs(&l.body, refs);
        }
        ast::Expr::NamedExpr(n) => {
            collect_expr_refs(&n.value, refs);
        }
        ast::Expr::Starred(s) => {
            collect_expr_refs(&s.value, refs);
        }
        ast::Expr::ListComp(l) => {
            collect_expr_refs(&l.elt, refs);
            collect_comp_generators(&l.generators, refs);
        }
        ast::Expr::SetComp(s) => {
            collect_expr_refs(&s.elt, refs);
            collect_comp_generators(&s.generators, refs);
        }
        ast::Expr::DictComp(d) => {
            collect_expr_refs(&d.key, refs);
            collect_expr_refs(&d.value, refs);
            collect_comp_generators(&d.generators, refs);
        }
        ast::Expr::GeneratorExp(g) => {
            collect_expr_refs(&g.elt, refs);
            collect_comp_generators(&g.generators, refs);
        }
        ast::Expr::Await(a) => {
            collect_expr_refs(&a.value, refs);
        }
        ast::Expr::Yield(y) => {
            if let Some(ref value) = y.value {
                collect_expr_refs(value, refs);
            }
        }
        ast::Expr::YieldFrom(y) => {
            collect_expr_refs(&y.value, refs);
        }
        ast::Expr::JoinedStr(j) => {
            for value in &j.values {
                collect_expr_refs(value, refs);
            }
        }
        ast::Expr::FormattedValue(f) => {
            collect_expr_refs(&f.value, refs);
            if let Some(ref spec) = f.format_spec {
                collect_expr_refs(spec, refs);
            }
        }
        ast::Expr::Constant(_) => {}
    }
}

fn collect_comp_generators(generators: &[ast::Comprehension], refs: &mut HashSet<String>) {
    for gen in generators {
        collect_expr_refs(&gen.iter, refs);
        for if_ in &gen.ifs {
            collect_expr_refs(if_, refs);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn check(src: &str) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, PathBuf::from("test.py"));
        check_unused_import(&source, &UnusedImportConfig::default())
    }

    fn check_with_path(src: &str, path: PathBuf) -> Vec<Diagnostic> {
        let source = SourceFile::from_string(src, path);
        check_unused_import(&source, &UnusedImportConfig::default())
    }

    #[test]
    fn unused_import() {
        let diags = check("import os\nx = 1\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("os"));
        assert_eq!(diags[0].rule_id, "readability-unused-import");
    }

    #[test]
    fn used_import_no_warning() {
        let diags = check("import os\nprint(os.path)\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn unused_from_import() {
        let diags = check("from os import path\nx = 1\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("path"));
    }

    #[test]
    fn used_from_import_no_warning() {
        let diags = check("from os import path\nprint(path)\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn aliased_import() {
        let diags = check("import numpy as np\nprint(np.array)\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn unused_aliased_import() {
        let diags = check("import numpy as np\nx = 1\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("np"));
    }

    #[test]
    fn type_checking_import_excluded() {
        let diags = check("from typing import TYPE_CHECKING\nif TYPE_CHECKING:\n    import os\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn multiple_imports_some_used() {
        let diags = check("import os\nimport sys\nprint(os.path)\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("sys"));
        assert!(!diags[0].message.contains("os"));
    }

    #[test]
    fn disabled_config() {
        let source = SourceFile::from_string("import os\nx = 1\n", PathBuf::from("test.py"));
        let config = UnusedImportConfig { enabled: false };
        let diags = check_unused_import(&source, &config);
        assert!(diags.is_empty());
    }

    #[test]
    fn parse_error_returns_empty() {
        let diags = check("def foo(:\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn init_file_excluded() {
        let diags = check_with_path("import os\nx = 1\n", PathBuf::from("__init__.py"));
        assert!(diags.is_empty());
    }

    #[test]
    fn star_import_not_flagged() {
        let diags = check("from os import *\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn from_import_with_alias() {
        let diags = check("from os import path as p\nx = 1\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("p"));
    }

    #[test]
    fn from_import_alias_used() {
        let diags = check("from os import path as p\nprint(p)\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn diagnostic_contains_source_line() {
        let diags = check("import os\nx = 1\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].source_line.as_deref(), Some("import os"));
    }

    #[test]
    fn attribute_access_counts_as_usage() {
        let diags = check("import os\nos.path.exists('x')\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn import_used_in_decorator() {
        let diags = check("import dataclasses\n@dataclasses.dataclass\nclass Foo:\n    x: int = 0\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn import_used_in_class_base() {
        let diags = check("from typing import NamedTuple\nclass Point(NamedTuple):\n    x: int\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn import_used_in_annotation() {
        let diags = check("from typing import Optional\ndef foo(x: Optional[int]) -> None:\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn import_used_in_default_param() {
        let diags = check("import os\ndef foo(path=os.path):\n    pass\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn import_used_in_return_annotation() {
        let diags = check("from typing import List\ndef foo() -> List[int]:\n    return []\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn kw_arg_not_counted_as_ref() {
        let diags = check("import os\nfoo(os=1)\n");
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("os"));
    }

    #[test]
    fn assign_target_attribute() {
        let diags = check("import os\nos.environ['KEY'] = 'value'\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn typing_type_checking_attr() {
        let diags = check("import typing\nif typing.TYPE_CHECKING:\n    import os\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn lambda_default_param() {
        let diags = check("import os\nf = lambda x=os.sep: x\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn init_file_subdir_not_confused() {
        let diags = check_with_path("import os\nx = 1\n", PathBuf::from("src/__init__.py"));
        assert!(diags.is_empty());
    }

    #[test]
    fn non_init_file_with_init_suffix() {
        let diags = check_with_path("import os\nx = 1\n", PathBuf::from("my__init__.py"));
        assert_eq!(diags.len(), 1);
    }
}
