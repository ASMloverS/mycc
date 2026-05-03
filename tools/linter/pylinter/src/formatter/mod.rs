use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn format_source(_source: &mut SourceFile, _config: &FormatConfig) -> Result<Vec<Diagnostic>, String> {
    Ok(Vec::new())
}
