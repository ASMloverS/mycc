#[derive(Clone, Debug, PartialEq)]
pub enum AstNodeKind {
    FunctionDef,
    ClassDef,
    Import,
    ImportFrom,
}

pub fn map_ast_to_lines(_ast: &rustpython_parser::ast::Mod, _lines: &mut [super::lines::CSTLine]) {
}
