use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_encoding(
    source: &mut SourceFile,
    _config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = source.content.as_str();
    let content = content.strip_prefix('\u{feff}').unwrap_or(content);
    let content = content.replace("\r\n", "\n").replace('\r', "\n");
    let had_newline = content.ends_with('\n');
    let lines: Vec<String> = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}
