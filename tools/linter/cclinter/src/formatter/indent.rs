use crate::common::source::SourceFile;
use crate::config::FormatConfig;

pub fn fix_indent(
    source: &mut SourceFile,
    config: &FormatConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    if config.use_tabs {
        return Ok(());
    }
    if config.indent_width == 0 {
        return Ok(());
    }
    let spaces = " ".repeat(config.indent_width);
    let content = &source.content;
    let had_newline = content.ends_with('\n');
    let lines: Vec<String> = content
        .lines()
        .map(|line| {
            let tab_count = line.chars().take_while(|&c| c == '\t').count();
            if tab_count == 0 {
                return line.to_string();
            }
            let rest: String = line.chars().skip(tab_count).collect();
            let replaced = spaces.repeat(tab_count);
            format!("{}{}", replaced, rest)
        })
        .collect();
    let result = lines.join("\n");
    source.content = if had_newline && !result.is_empty() {
        format!("{}\n", result)
    } else {
        result
    };
    Ok(())
}
