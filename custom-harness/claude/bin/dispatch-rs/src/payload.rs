use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;
use serde::Serialize;

use crate::{frontmatter, prompt, registry};

#[derive(Serialize)]
pub struct Payload {
    pub subagent_type: &'static str,
    pub description: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_in_background: Option<bool>,
}

#[derive(Serialize)]
pub struct Output {
    pub mode: &'static str,
    pub payloads: Vec<Payload>,
}

type MdCache = HashMap<String, (HashMap<String, String>, String)>;

pub fn build_payload(
    registry: &registry::Registry,
    harness: &PathBuf,
    name_token: &str,
    user_prompt: &str,
    model: Option<&str>,
    bg: bool,
    inline: bool,
    md_cache: &mut MdCache,
) -> Result<Payload> {
    let (type_, name, entry) = registry::resolve_name(registry, name_token)?;
    // Normalize forward slashes so display() matches Python's Path.__str__() on Windows
    let md_path = harness.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
    anyhow::ensure!(md_path.exists(), "MD not found: {}", md_path.display());

    let (fm, body) = if let Some(cached) = md_cache.get(&entry.path) {
        cached.clone()
    } else {
        let parsed = frontmatter::parse_md(&md_path)?;
        md_cache.insert(entry.path.clone(), parsed.clone());
        parsed
    };

    anyhow::ensure!(!body.trim().is_empty(), "Empty MD body: {}", md_path.display());

    let prompt_str = if inline {
        prompt::assemble_inline(&type_, &name, &body, &fm, user_prompt, harness)
    } else {
        prompt::assemble_thin(&type_, &name, &fm, user_prompt, &md_path, harness)
    };

    let effective_model = model.map(str::to_string).or_else(|| fm.get("model").cloned());

    Ok(Payload {
        subagent_type: "general-purpose",
        description: entry.desc.chars().take(50).collect(),
        prompt: prompt_str,
        model: effective_model,
        run_in_background: if bg { Some(true) } else { None },
    })
}

pub fn print_help(registry: &registry::Registry, harness: &PathBuf, name: Option<&str>) {
    if let Some(n) = name {
        for (type_, section) in registry {
            if let Some(entry) = section.get(n) {
                let md_path = harness.join(entry.path.replace('/', std::path::MAIN_SEPARATOR_STR));
                let fm = md_path.exists()
                    .then(|| frontmatter::parse_md(&md_path).ok().map(|(f, _)| f))
                    .flatten()
                    .unwrap_or_default();
                println!("[{type_}] {n}");
                println!("  desc: {}", entry.desc);
                for (k, v) in &fm { println!("  {k}: {v}"); }
                return;
            }
        }
        eprintln!("dispatch: Unknown name: {n:?}");
        std::process::exit(2);
    }
    println!("Usage: dispatch <name|type:name> <prompt> [--model M] [--bg] [--help [name]]\n");
    let mut sections: Vec<_> = registry.iter().collect();
    sections.sort_by_key(|(k, _)| k.as_str());
    for (type_, section) in &sections {
        println!("[{type_}]");
        let mut items: Vec<_> = section.iter().collect();
        items.sort_by_key(|(k, _)| k.as_str());
        for (n, e) in items {
            println!("  {n:<20} {}", e.desc);
        }
        println!();
    }
}
