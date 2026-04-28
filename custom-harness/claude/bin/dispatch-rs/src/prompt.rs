use std::collections::HashMap;
use std::path::Path;

/// Thin-pointer prompt. Mirrors Python assemble_thin_prompt().
pub fn assemble_thin(
    type_: &str,
    name: &str,
    fm: &HashMap<String, String>,
    user_prompt: &str,
    md_path: &Path,
    harness_dir: &Path,
) -> String {
    let tools = fm.get("tools").map(|s| s.as_str()).unwrap_or("");
    let tool_hint = if tools.is_empty() {
        "TOOL_HINT:".to_string()
    } else {
        format!("TOOL_HINT: {tools}")
    };
    // Match Python exactly: harness_line always present, trailing \n on tool_hint line
    format!(
        "You are a one-shot subagent. Read the file at DEFINITION_FILE as your\n\
         literal instructions, then process the User Input.\n\n\
         DEFINITION_FILE: {md}\n\
         HARNESS_DIR: {hd}\n\
         TYPE: {type_}/{name}\n\
         {tool_hint}\n\
         \n---\n## User Input\n\
         {user_prompt}",
        md = md_path.display(),
        hd = harness_dir.display(),
    )
}

/// Inline (legacy) prompt. Mirrors Python assemble_prompt().
pub fn assemble_inline(
    type_: &str,
    name: &str,
    body: &str,
    fm: &HashMap<String, String>,
    user_prompt: &str,
    harness_dir: &Path,
) -> String {
    let tools = fm.get("tools").map(|s| s.as_str()).unwrap_or("");
    let tool_hint = if tools.is_empty() {
        String::new()
    } else {
        format!(
            "\nTool access guidance (soft): originally authored for tools = [{tools}].\
             \nPrefer those; avoid unrelated tools.\n"
        )
    };
    let harness_hint = format!("\nHARNESS_DIR = {}\n", harness_dir.display());
    format!(
        "You are a one-shot general-purpose subagent executing the definition below.\n\
         Follow it literally.\n\n\
         <!-- BEGIN: {type_}/{name} -->\n\
         {body_stripped}\n\
         <!-- END -->\n\
         {tool_hint}\
         {harness_hint}\
         \n---\n## User Input\n\
         {user_prompt}",
        body_stripped = body.trim(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn hd() -> PathBuf { PathBuf::from(r"C:\harness") }
    fn md() -> PathBuf { PathBuf::from(r"C:\harness\agents\foo.md") }

    #[test]
    fn thin_has_definition_file() {
        let fm = HashMap::new();
        let p = assemble_thin("agents", "foo", &fm, "test", &md(), &hd());
        assert!(p.contains("DEFINITION_FILE:"));
        assert!(!p.contains("<!-- BEGIN:"));
    }

    #[test]
    fn inline_has_begin_marker() {
        let fm = HashMap::new();
        let p = assemble_inline("agents", "foo", "body text", &fm, "test", &hd());
        assert!(p.contains("<!-- BEGIN: agents/foo -->"));
        assert!(!p.contains("DEFINITION_FILE:"));
    }

    #[test]
    fn thin_tool_hint_empty() {
        let fm = HashMap::new();
        let p = assemble_thin("agents", "foo", &fm, "prompt", &md(), &hd());
        assert!(p.contains("TOOL_HINT:\n"));
    }

    #[test]
    fn thin_tool_hint_present() {
        let mut fm = HashMap::new();
        fm.insert("tools".to_string(), "Bash, Read".to_string());
        let p = assemble_thin("agents", "foo", &fm, "prompt", &md(), &hd());
        assert!(p.contains("TOOL_HINT: Bash, Read"));
    }
}
