pub mod encoding;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn format_source(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<Vec<Diagnostic>, String> {
    encoding::fix_encoding(source, config).map_err(|e| e.to_string())?;
    Ok(Vec::new())
}
