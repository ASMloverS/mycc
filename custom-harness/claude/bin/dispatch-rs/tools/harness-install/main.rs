use std::collections::HashSet;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use dialoguer::MultiSelect;

struct SourceSkill {
    name: String,
    source_path: PathBuf,
    description: String,
}

enum Action {
    Install(SourceSkill),
    Uninstall(String),
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Err(e) = run(args) {
        let msg = e.to_string();
        eprintln!("harness-install: {msg}");
        let code = if msg.contains("not found") || msg.contains("Unknown skill") { 2 } else { 1 };
        std::process::exit(code);
    }
}

fn run(args: Vec<String>) -> Result<()> {
    let list_only = args.iter().any(|a| a == "--list");
    let names: Vec<&str> = args.iter()
        .filter(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .collect();

    let source_dir = agents_skills_dir()?;
    let harness = harness_dir()?;

    let items = scan_source_skills(&source_dir)
        .with_context(|| format!("scanning {}", source_dir.display()))?;
    let installed = installed_set(&harness)?;

    if list_only {
        print_state(&items, &installed);
        return Ok(());
    }

    if !names.is_empty() {
        for name in &names {
            if !items.iter().any(|s| s.name == *name) {
                anyhow::bail!("Unknown skill {:?}. Run --list to see available skills.", name);
            }
        }
        for name in &names {
            if installed.contains(*name) {
                println!("already installed: {name}");
                continue;
            }
            let skill = items.iter().find(|s| s.name == *name).unwrap();
            install_skill(&harness, skill)?;
            println!("installed: {name}");
        }
        return Ok(());
    }

    // Interactive TUI
    let actions = prompt_selection(&items, &installed)?;
    if actions.is_empty() {
        println!("No changes.");
        return Ok(());
    }
    for action in actions {
        match action {
            Action::Install(skill) => {
                install_skill(&harness, &skill)?;
                println!("installed: {}", skill.name);
            }
            Action::Uninstall(name) => {
                uninstall_skill(&harness, &name)?;
                println!("uninstalled: {name}");
            }
        }
    }
    Ok(())
}

fn agents_skills_dir() -> Result<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("cannot determine home directory (USERPROFILE / HOME not set)")?;
    let dir = PathBuf::from(home).join(".agents").join("skills");
    anyhow::ensure!(dir.exists(), "~/.agents/skills/ not found: {}", dir.display());
    Ok(dir)
}

fn harness_dir() -> Result<PathBuf> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .context("cannot determine home directory")?;
    Ok(PathBuf::from(home).join(".claude").join("custom-harness"))
}

fn scan_source_skills(dir: &Path) -> Result<Vec<SourceSkill>> {
    let mut items = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let p = entry.path();
        // Follow symlinks to check if it's a directory
        let meta = match std::fs::metadata(&p) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if !meta.is_dir() {
            continue;
        }
        let skill_md = p.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let name = p.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let description = read_skill_description(&skill_md).unwrap_or_default();
        items.push(SourceSkill { name, source_path: p, description });
    }
    items.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(items)
}

/// Extract `description:` from SKILL.md frontmatter using serde_yaml so that
/// YAML block scalars (`>` folded, `|` literal) are resolved correctly.
fn read_skill_description(skill_md: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(skill_md)?;
    let text = raw.replace("\r\n", "\n");
    let rest = match text.strip_prefix("---") {
        None => return Ok(String::new()),
        Some(s) => s,
    };
    let rest = match rest.strip_prefix('\n').or_else(|| rest.strip_prefix(" \n")) {
        None => return Ok(String::new()),
        Some(s) => s,
    };
    // Find closing ---
    let Some(end) = rest.find("\n---") else { return Ok(String::new()) };
    let fm_raw = &rest[..end];
    let val: serde_yaml::Value = serde_yaml::from_str(fm_raw).unwrap_or(serde_yaml::Value::Null);
    let desc = val.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        // Collapse internal newlines produced by folded/literal scalars
        .replace('\n', " ")
        .trim()
        .to_string();
    Ok(desc)
}

fn installed_set(harness: &Path) -> Result<HashSet<String>> {
    let yaml_path = harness.join("registry.yaml");
    anyhow::ensure!(yaml_path.exists(), "registry.yaml not found: {}", yaml_path.display());
    let text = std::fs::read_to_string(&yaml_path)?;
    Ok(parse_installed_from_yaml(&text))
}

fn parse_installed_from_yaml(text: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    let mut in_skills = false;
    for line in text.lines() {
        if line == "skills:" {
            in_skills = true;
            continue;
        }
        if in_skills {
            if !line.starts_with(' ') && !line.is_empty() {
                break; // next top-level key
            }
            if let Some(rest) = line.strip_prefix("  ") {
                if let Some(col) = rest.find(':') {
                    let name = rest[..col].trim().to_string();
                    if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                        set.insert(name);
                    }
                }
            }
        }
    }
    set
}

fn print_state(items: &[SourceSkill], installed: &HashSet<String>) {
    println!("{:<22} {:<10} {}", "name", "status", "description");
    println!("{}", "-".repeat(70));
    for s in items {
        let tag = if installed.contains(&s.name) { "[installed]" } else { "[available]" };
        println!("{:<22} {:<10}  {}", s.name, tag, truncate_desc(&s.description, 50));
    }
}

fn prompt_selection(items: &[SourceSkill], installed: &HashSet<String>) -> Result<Vec<Action>> {
    let labels: Vec<String> = items.iter().map(|s| {
        let tag = if installed.contains(&s.name) { "[已装]" } else { "      " };
        format!("{tag} {:<22} {}", s.name, truncate_desc(&s.description, 55))
    }).collect();
    let defaults: Vec<bool> = items.iter().map(|s| installed.contains(&s.name)).collect();

    let chosen = MultiSelect::new()
        .with_prompt("选择要安装/卸载的 skills (空格=切换, Enter=确认)")
        .items(&labels)
        .defaults(&defaults)
        .interact()
        .context("TUI interaction failed")?;

    let chosen_set: HashSet<usize> = chosen.into_iter().collect();
    let mut actions = Vec::new();
    for (i, skill) in items.iter().enumerate() {
        let was = defaults[i];
        let now = chosen_set.contains(&i);
        match (was, now) {
            (false, true) => actions.push(Action::Install(SourceSkill {
                name: skill.name.clone(),
                source_path: skill.source_path.clone(),
                description: skill.description.clone(),
            })),
            (true, false) => actions.push(Action::Uninstall(skill.name.clone())),
            _ => {}
        }
    }
    Ok(actions)
}

fn install_skill(harness: &Path, skill: &SourceSkill) -> Result<()> {
    let link = harness.join("skills").join(&skill.name);
    if link.exists() || link.symlink_metadata().is_ok() {
        // Exists — is it already a link/junction?
        let meta = std::fs::symlink_metadata(&link)?;
        if meta.is_dir() && !is_symlink_or_junction(&link) {
            anyhow::bail!("skills/{} exists and is a real directory; refusing to overwrite", skill.name);
        }
        // Already a link — treat as idempotent
        return Ok(());
    }
    make_link(&skill.source_path, &link)?;
    let desc = truncate_desc(&skill.description, 100);
    registry_insert(harness, &skill.name, &desc)?;
    Ok(())
}

fn uninstall_skill(harness: &Path, name: &str) -> Result<()> {
    let link = harness.join("skills").join(name);
    if link.symlink_metadata().is_ok() {
        remove_link(&link).with_context(|| format!("removing link skills/{name}"))?;
    }
    registry_remove(harness, name)?;
    Ok(())
}

fn is_symlink_or_junction(p: &Path) -> bool {
    match std::fs::symlink_metadata(p) {
        Ok(m) => m.file_type().is_symlink() || is_junction(p),
        Err(_) => false,
    }
}

#[cfg(windows)]
fn is_junction(p: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    // FILE_ATTRIBUTE_REPARSE_POINT = 0x400
    match std::fs::symlink_metadata(p) {
        Ok(m) => (m.file_attributes() & 0x400) != 0,
        Err(_) => false,
    }
}

#[cfg(not(windows))]
fn is_junction(_p: &Path) -> bool {
    false
}

#[cfg(unix)]
fn make_link(target: &Path, link: &Path) -> Result<()> {
    std::os::unix::fs::symlink(target, link)
        .with_context(|| format!("symlink {} -> {}", link.display(), target.display()))
}

#[cfg(windows)]
fn make_link(target: &Path, link: &Path) -> Result<()> {
    use std::os::windows::fs::symlink_dir;
    match symlink_dir(target, link) {
        Ok(()) => Ok(()),
        Err(e) if e.raw_os_error() == Some(1314) => {
            // ERROR_PRIVILEGE_NOT_HELD — fall back to directory junction
            let st = std::process::Command::new("cmd")
                .args(["/c", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .output()
                .context("spawning mklink /J")?;
            anyhow::ensure!(st.status.success(),
                "mklink /J failed: {}", String::from_utf8_lossy(&st.stderr));
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!(e)
            .context(format!("symlink_dir {} -> {}", link.display(), target.display()))),
    }
}

fn remove_link(link: &Path) -> Result<()> {
    anyhow::ensure!(
        is_symlink_or_junction(link),
        "{} is not a symlink or junction; refusing to remove", link.display()
    );
    std::fs::remove_dir(link)
        .with_context(|| format!("remove_dir {}", link.display()))
}

fn registry_insert(harness: &Path, name: &str, desc: &str) -> Result<()> {
    let yaml_path = harness.join("registry.yaml");
    let text = std::fs::read_to_string(&yaml_path)?;

    // Idempotent: skip if already present
    if parse_installed_from_yaml(&text).contains(name) {
        return Ok(());
    }

    let escaped = desc.replace('\\', "\\\\").replace('"', "\\\"");
    let new_line = format!(
        "  {name}: {{ path: skills/{name}/SKILL.md, desc: \"{escaped}\" }}\n"
    );

    let mut result = String::with_capacity(text.len() + new_line.len());
    let mut in_skills = false;
    let mut inserted = false;

    for line in text.lines() {
        if !inserted {
            if line == "skills:" {
                in_skills = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }
            if in_skills && !line.starts_with(' ') && !line.is_empty() {
                // Hit next top-level key — insert before it
                result.push_str(&new_line);
                inserted = true;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    // If skills: was the last section (no trailing top-level key after it)
    if in_skills && !inserted {
        result.push_str(&new_line);
    }

    std::fs::write(&yaml_path, result)?;
    Ok(())
}

fn registry_remove(harness: &Path, name: &str) -> Result<()> {
    let yaml_path = harness.join("registry.yaml");
    let text = std::fs::read_to_string(&yaml_path)?;

    let prefix = format!("  {name}:");
    let mut in_skills = false;
    let mut result = String::with_capacity(text.len());

    for line in text.lines() {
        if line == "skills:" {
            in_skills = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }
        if in_skills && !line.starts_with(' ') && !line.is_empty() {
            in_skills = false;
        }
        if in_skills && line.starts_with(&prefix) {
            continue; // drop this line
        }
        result.push_str(line);
        result.push('\n');
    }

    std::fs::write(&yaml_path, result)?;
    Ok(())
}

fn truncate_desc(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    // Truncate at sentence boundary first
    if let Some(pos) = s[..max].rfind(". ") {
        return s[..pos + 1].to_string();
    }
    // Fall back to word boundary
    if let Some(pos) = s[..max].rfind(' ') {
        return format!("{}…", &s[..pos]);
    }
    format!("{}…", &s[..max])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn truncate_at_sentence() {
        let s = "First sentence. Second sentence goes longer than limit.";
        assert_eq!(truncate_desc(s, 30), "First sentence.");
    }

    #[test]
    fn truncate_at_word() {
        let s = "No periods in this long description text that overflows";
        let t = truncate_desc(s, 20);
        assert!(t.len() <= 22); // "…" may add a byte or two
        assert!(!t.contains("overflows"));
    }

    #[test]
    fn truncate_short_passthrough() {
        let s = "Short text";
        assert_eq!(truncate_desc(s, 100), "Short text");
    }

    #[test]
    fn parse_installed_skills() {
        let yaml = "agents:\n  foo: {path: a, desc: d}\nskills:\n  bar: {path: b, desc: e}\n  baz: {path: c, desc: f}\ncommands: {}\n";
        let set = parse_installed_from_yaml(yaml);
        assert!(set.contains("bar"));
        assert!(set.contains("baz"));
        assert!(!set.contains("foo"));
    }

    #[test]
    fn parse_installed_stops_at_next_section() {
        let yaml = "skills:\n  alpha: {path: a, desc: d}\ncommands: {}\nagents:\n  beta: {path: b, desc: e}\n";
        let set = parse_installed_from_yaml(yaml);
        assert!(set.contains("alpha"));
        assert!(!set.contains("beta"));
    }

    #[test]
    fn registry_insert_and_remove_roundtrip() {
        let dir = TempDir::new().unwrap();
        let yaml_path = dir.path().join("registry.yaml");
        let original = "agents:\n  foo: {path: agents/foo.md, desc: \"Foo\"}\n\nskills:\n  bar: {path: skills/bar/SKILL.md, desc: \"Bar\"}\n\ncommands: {}\n";
        fs::write(&yaml_path, original).unwrap();

        let harness = dir.path();
        registry_insert(harness, "newskill", "A new skill").unwrap();

        let after_insert = fs::read_to_string(&yaml_path).unwrap();
        let installed = parse_installed_from_yaml(&after_insert);
        assert!(installed.contains("bar"));
        assert!(installed.contains("newskill"));

        // Original lines preserved
        assert!(after_insert.contains("  foo: {path: agents/foo.md, desc: \"Foo\"}"));
        assert!(after_insert.contains("  bar: {path: skills/bar/SKILL.md, desc: \"Bar\"}"));

        registry_remove(harness, "newskill").unwrap();
        let after_remove = fs::read_to_string(&yaml_path).unwrap();
        let installed2 = parse_installed_from_yaml(&after_remove);
        assert!(!installed2.contains("newskill"));
        assert!(installed2.contains("bar"));
    }

    #[test]
    fn registry_insert_idempotent() {
        let dir = TempDir::new().unwrap();
        let yaml_path = dir.path().join("registry.yaml");
        let original = "skills:\n  existing: {path: skills/existing/SKILL.md, desc: \"Existing\"}\n\ncommands: {}\n";
        fs::write(&yaml_path, original).unwrap();

        registry_insert(dir.path(), "existing", "Updated desc").unwrap();
        let result = fs::read_to_string(&yaml_path).unwrap();
        // Should not have duplicated the entry
        assert_eq!(result.matches("existing:").count(), 1);
    }

    #[test]
    fn scan_skips_entries_without_skill_md() {
        let dir = TempDir::new().unwrap();
        // real skill dir
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "---\nname: my-skill\ndescription: A skill\n---\nbody\n").unwrap();
        // dir without SKILL.md (like superpowers)
        let nested = dir.path().join("nested-container");
        fs::create_dir(&nested).unwrap();
        fs::create_dir(nested.join("sub-skill")).unwrap();

        let items = scan_source_skills(dir.path()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "my-skill");
        assert_eq!(items[0].description, "A skill");
    }
}
