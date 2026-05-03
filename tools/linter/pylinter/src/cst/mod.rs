pub mod astmap;
pub mod generate;
pub mod lines;
pub mod tokens;

pub use lines::{CSTLine, IndentInfo};
pub use tokens::{LocatedToken, tokenize_source};

use rustpython_parser::ast::Mod;

pub struct CSTSource {
    pub lines: Vec<CSTLine>,
    pub ast: Mod,
    pub source: String,
}

impl CSTSource {
    pub fn parse(source: &str) -> Result<Self, String> {
        let tokens = tokenize_source(source)?;
        let ast = rustpython_parser::parse(source, rustpython_parser::Mode::Module, "<input>")
            .map_err(|e| e.to_string())?;
        let lines = lines::build_lines(&tokens, source);
        Ok(Self {
            lines,
            ast,
            source: source.to_string(),
        })
    }
}
