use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub path: String,
    pub desc: String,
}

pub type Section = HashMap<String, Entry>;
pub type Registry = HashMap<String, Section>;

#[derive(Serialize, Deserialize)]
struct Cache {
    mtime: u64,
    registry: Registry,
}

pub fn load_registry(harness_dir: &Path) -> Result<Registry> {
    let yaml_path = harness_dir.join("registry.yaml");
    anyhow::ensure!(yaml_path.exists(), "registry.yaml not found: {}", yaml_path.display());
    let mtime = file_mtime(&yaml_path)?;
    let cache_path = harness_dir.join(".cache").join("registry.bin");
    if let Ok(reg) = try_cache(&cache_path, mtime) {
        return Ok(reg);
    }
    let text = std::fs::read_to_string(&yaml_path).context("read registry.yaml")?;
    let reg = parse_yaml(&text).context("parse registry.yaml")?;
    save_cache(&cache_path, mtime, &reg);
    Ok(reg)
}

fn file_mtime(p: &Path) -> Result<u64> {
    let t = std::fs::metadata(p)?.modified()?;
    Ok(t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs())
}

fn try_cache(path: &Path, mtime: u64) -> Result<Registry> {
    let data = std::fs::read(path)?;
    let c: Cache = bincode::deserialize(&data)?;
    anyhow::ensure!(c.mtime == mtime, "stale");
    Ok(c.registry)
}

fn save_cache(path: &Path, mtime: u64, reg: &Registry) {
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    if let Ok(data) = bincode::serialize(&Cache { mtime, registry: reg.clone() }) {
        let _ = std::fs::write(path, data);
    }
}

fn parse_yaml(text: &str) -> Result<Registry> {
    let raw: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(text)?;
    let mut reg = Registry::new();
    for (section, val) in raw {
        // commands: {} → empty HashMap; other sections → HashMap<String, Entry>
        let section_map: Section = serde_yaml::from_value(val).unwrap_or_default();
        reg.insert(section, section_map);
    }
    Ok(reg)
}

pub fn resolve_name(registry: &Registry, token: &str) -> Result<(String, String, Entry)> {
    if let Some((type_, name)) = token.split_once(':') {
        if !name.is_empty() {
            return registry
                .get(type_)
                .and_then(|s| s.get(name))
                .map(|e| (type_.to_string(), name.to_string(), e.clone()))
                .ok_or_else(|| anyhow::anyhow!("Not found: {:?}", token));
        }
        return resolve_fuzzy(registry, type_); // trailing colon → strip and fuzzy
    }
    resolve_fuzzy(registry, token)
}

fn resolve_fuzzy(registry: &Registry, name: &str) -> Result<(String, String, Entry)> {
    let mut matches: Vec<(String, String, Entry)> = registry
        .iter()
        .filter_map(|(t, s)| s.get(name).map(|e| (t.clone(), name.to_string(), e.clone())))
        .collect();
    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => anyhow::bail!("Unknown name: {:?}. Run --help to list registered items.", name),
        _ => {
            let opts: Vec<_> = matches.iter().map(|(t, n, _)| format!("{t}:{n}")).collect();
            anyhow::bail!("Ambiguous name {:?}. Use: {}", name, opts.join(" / "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = "agents:\n  foo: {path: agents/foo.md, desc: \"Foo agent\"}\nskills:\n  bar: {path: skills/bar.md, desc: \"Bar skill\"}\ncommands: {}\n";

    #[test]
    fn parse_sections() {
        let reg = parse_yaml(SAMPLE_YAML).unwrap();
        assert!(reg.contains_key("agents"));
        assert!(reg.contains_key("skills"));
        assert!(reg.contains_key("commands"));
        assert!(reg["commands"].is_empty());
    }

    #[test]
    fn resolve_direct() {
        let reg = parse_yaml(SAMPLE_YAML).unwrap();
        let (t, n, e) = resolve_name(&reg, "agents:foo").unwrap();
        assert_eq!(t, "agents");
        assert_eq!(n, "foo");
        assert_eq!(e.desc, "Foo agent");
    }

    #[test]
    fn resolve_fuzzy_ok() {
        let reg = parse_yaml(SAMPLE_YAML).unwrap();
        let (t, n, _) = resolve_name(&reg, "foo").unwrap();
        assert_eq!(t, "agents");
        assert_eq!(n, "foo");
    }

    #[test]
    fn resolve_trailing_colon() {
        let reg = parse_yaml(SAMPLE_YAML).unwrap();
        let (_, n, _) = resolve_name(&reg, "foo:").unwrap();
        assert_eq!(n, "foo");
    }

    #[test]
    fn resolve_unknown() {
        let reg = parse_yaml(SAMPLE_YAML).unwrap();
        assert!(resolve_name(&reg, "nonexistent").is_err());
    }
}
