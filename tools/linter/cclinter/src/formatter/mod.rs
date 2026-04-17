pub mod alignment;
pub mod blank_lines;
pub mod braces;
pub mod comments;
pub mod encoding;
pub mod include_sort;
pub mod indent;
pub mod line_length;
pub mod pointer_style;
pub mod spacing;

use crate::common::diag::Diagnostic;
use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn format_source(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error>> {
    encoding::fix_encoding(source, config)?;
    indent::fix_indent(source, config)?;
    spacing::fix_spacing(source, config)?;
    braces::fix_braces(source, config)?;
    blank_lines::fix_blank_lines(source, config)?;
    comments::fix_comments(source, config)?;
    pointer_style::fix_pointer_style(source, config)?;
    line_length::fix_line_length(source, config)?;
    alignment::fix_alignment(source, config)?;
    include_sort::fix_include_sort(source, config)?;
    Ok(vec![])
}
