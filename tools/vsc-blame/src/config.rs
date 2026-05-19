use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub vcs: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub no_color: Option<bool>,
    #[serde(default)]
    pub author_aliases: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub weights: Option<Weights>,
    #[serde(default)]
    pub defaults: Option<Defaults>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Weights {
    #[serde(default)]
    pub commit_count: Option<f64>,
    #[serde(default)]
    pub recency: Option<f64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Defaults {
    #[serde(default)]
    pub blame_summary: Option<bool>,
    #[serde(default)]
    pub blame_all: Option<bool>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read config {}: {}", path.display(), e))?;
        serde_yaml::from_str(&content).map_err(|e| format!("invalid config YAML: {}", e))
    }

    pub fn resolve_vcs(&self, cli: Option<&str>) -> Option<String> {
        if let Some(s) = cli {
            return Some(s.to_string());
        }
        if let Ok(val) = std::env::var("VSC_BLAME_VCS") {
            return Some(val);
        }
        self.vcs.clone()
    }

    pub fn resolve_format(&self, cli: Option<&str>) -> String {
        if let Some(s) = cli {
            return s.to_string();
        }
        if let Ok(val) = std::env::var("VSC_BLAME_FORMAT") {
            return val;
        }
        self.format.clone().unwrap_or_else(|| "text".to_string())
    }

    pub fn resolve_no_color(&self, cli_flag: bool) -> bool {
        if cli_flag {
            return true;
        }
        if let Ok(val) = std::env::var("VSC_BLAME_NO_COLOR") {
            return val == "1" || val == "true";
        }
        self.no_color.unwrap_or(false)
    }

    pub fn resolve_output(&self, cli: Option<&str>) -> Option<String> {
        if let Some(s) = cli {
            return Some(s.to_string());
        }
        if let Ok(val) = std::env::var("VSC_BLAME_OUTPUT") {
            return Some(val);
        }
        self.output.clone()
    }

    pub fn resolve_config_path(cli: Option<&str>) -> std::path::PathBuf {
        if let Some(p) = cli {
            return std::path::PathBuf::from(p);
        }
        std::path::PathBuf::from(".vsc-blame.yaml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_missing_config() {
        let cfg = Config::load(Path::new("/nonexistent/.vsc-blame.yaml")).unwrap();
        assert!(cfg.vcs.is_none());
    }

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
vcs: git
format: json
no_color: true
author_aliases:
  alice:
    - al
    - alice@old.com
"#;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), yaml).unwrap();
        let cfg = Config::load(tmp.path()).unwrap();
        assert_eq!(cfg.vcs.as_deref(), Some("git"));
        assert_eq!(cfg.format.as_deref(), Some("json"));
        assert_eq!(cfg.no_color, Some(true));
        assert!(cfg.author_aliases.contains_key("alice"));
    }

    #[test]
    fn test_load_invalid_config() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), "{{invalid yaml").unwrap();
        assert!(Config::load(tmp.path()).is_err());
    }

    #[test]
    fn test_resolve_format_priority() {
        let cfg = Config::default();
        assert_eq!(cfg.resolve_format(Some("html")), "html");
    }

    #[test]
    fn test_resolve_format_default() {
        let cfg = Config::default();
        assert_eq!(cfg.resolve_format(None), "text");
    }

    #[test]
    fn test_resolve_no_color_default() {
        let cfg = Config::default();
        assert!(!cfg.resolve_no_color(false));
        assert!(cfg.resolve_no_color(true));
    }
}
