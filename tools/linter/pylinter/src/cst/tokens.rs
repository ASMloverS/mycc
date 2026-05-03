use rustpython_parser::lexer::lex;
use rustpython_parser::text_size::TextSize;
use rustpython_parser::{Mode, Tok};

#[derive(Clone, Debug)]
pub struct LocatedToken {
    pub tok: Tok,
    pub start: TextSize,
    pub end: TextSize,
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

pub fn tokenize_source(source: &str) -> Result<Vec<LocatedToken>, String> {
    let line_map = LineMap::new(source);
    let tokens: Vec<_> = lex(source, Mode::Module).collect();

    let mut result = Vec::new();
    for item in tokens {
        let (tok, range) = item.map_err(|e| format!("{e:?}"))?;
        if matches!(tok, Tok::EndOfFile) {
            continue;
        }
        let (start_line, start_col) = line_map.position(range.start());
        let (end_line, end_col) = line_map.position(range.end());
        result.push(LocatedToken {
            tok,
            start: range.start(),
            end: range.end(),
            start_line,
            start_col,
            end_line,
            end_col,
        });
    }
    Ok(result)
}

struct LineMap {
    line_starts: Vec<TextSize>,
}

impl LineMap {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![TextSize::from(0)];
        let bytes = source.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\n' => {
                    line_starts.push(TextSize::from((i + 1) as u32));
                    i += 1;
                }
                b'\r' => {
                    if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        line_starts.push(TextSize::from((i + 2) as u32));
                        i += 2;
                    } else {
                        line_starts.push(TextSize::from((i + 1) as u32));
                        i += 1;
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
        Self { line_starts }
    }

    /// Returns (1-based line, 0-based byte column) for a byte offset.
    fn position(&self, offset: TextSize) -> (usize, usize) {
        let line_idx = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let line = line_idx + 1; // 1-based
        let col: u32 = (offset - self.line_starts[line_idx]).into();
        (line, col as usize)
    }
}
