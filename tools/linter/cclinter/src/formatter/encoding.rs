use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_encoding(
    _source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
