pub mod blank_lines;
pub mod comment_style;
pub mod encoding;
pub mod import_sort;
pub mod indent;
pub mod line_length;
pub mod trailing_ws;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::FormatConfig;
use crate::cst::CSTSource;

pub fn format_source(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<Vec<Diagnostic>, String> {
    encoding::fix_encoding(source, config).map_err(|e| e.to_string())?;
    if let Ok(mut cst) = CSTSource::parse(&source.content) {
        indent::fix_indent(&mut cst, config).map_err(|e| e.to_string())?;
        blank_lines::fix_blank_lines(&mut cst, config).map_err(|e| e.to_string())?;
        import_sort::fix_import_sort(&mut cst, config).map_err(|e| e.to_string())?;
        comment_style::fix_comment_style(&mut cst, config).map_err(|e| e.to_string())?;
        line_length::fix_line_length(&mut cst, config).map_err(|e| e.to_string())?;
        trailing_ws::fix_trailing_ws(&mut cst, config).map_err(|e| e.to_string())?;
        source.content = cst.regenerate();
    }
    Ok(Vec::new())
}
