pub mod encoding;
pub mod indent;
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
        trailing_ws::fix_trailing_ws(&mut cst, config).map_err(|e| e.to_string())?;
        source.content = cst.regenerate();
    }
    Ok(Vec::new())
}
