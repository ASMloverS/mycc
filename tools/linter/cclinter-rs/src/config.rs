use std::path::Path;

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub verbose: Option<u8>,
    pub line_length: Option<usize>,
    pub output: Option<String>,
    pub recursive: Option<bool>,
    pub quiet: Option<bool>,
    pub repository: Option<String>,
    pub root: Option<String>,
    pub include_order: Option<String>,
    pub filters: Option<FilterConfig>,
    pub extensions: Option<ExtensionConfig>,
    pub exclude: Option<ExcludeConfig>,
    pub fix: Option<FixConfig>,
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default)]
pub struct FilterConfig {
    pub add: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(default)]
pub struct ExtensionConfig {
    pub headers: Option<Vec<String>>,
    pub sources: Option<Vec<String>>,
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            headers: Some(vec![
                "h".into(),
                "hh".into(),
                "hpp".into(),
                "hxx".into(),
                "h++".into(),
                "cuh".into(),
            ]),
            sources: Some(vec![
                "c".into(),
                "cc".into(),
                "cpp".into(),
                "cxx".into(),
                "c++".into(),
                "cu".into(),
            ]),
        }
    }
}

#[derive(Debug, serde::Deserialize, Default)]
#[serde(default)]
pub struct ExcludeConfig {
    pub files: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(default)]
pub struct FixConfig {
    pub trailing_whitespace: Option<bool>,
    pub utf8_bom: Option<bool>,
    pub crl: Option<bool>,
    pub block_comments: Option<bool>,
}

impl Default for FixConfig {
    fn default() -> Self {
        Self {
            trailing_whitespace: Some(true),
            utf8_bom: Some(true),
            crl: Some(true),
            block_comments: Some(true),
        }
    }
}

impl Config {
    pub fn load(dir: &Path) -> Result<Option<Self>, crate::error::LintError> {
        let mut current = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(dir)
        };
        loop {
            let candidate = current.join("cclinter-rs.toml");
            if candidate.is_file() {
                let raw = std::fs::read_to_string(&candidate)?;
                let cfg: Config = toml::from_str(&raw)?;
                return Ok(Some(cfg));
            }
            if !current.pop() {
                return Ok(None);
            }
        }
    }

    pub fn with_cli_overrides(self, _cli: &()) -> Self {
        self
    }

    pub fn effective_line_length(&self) -> usize {
        self.line_length.unwrap_or(80)
    }

    pub fn effective_verbose(&self) -> u8 {
        self.verbose.unwrap_or(1)
    }

    pub fn effective_output(&self) -> &str {
        self.output.as_deref().unwrap_or("emacs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
verbose = 3
line_length = 120
output = "vs7"
recursive = true
quiet = true
repository = "/path/to/repo"
root = "src"
include_order = "standardcfirst"

[filters]
add = ["-build/include_alpha", "+build/include"]

[extensions]
headers = ["h", "hpp"]
sources = ["c", "cpp"]

[exclude]
files = ["build/**", "third_party/**"]

[fix]
trailing_whitespace = true
utf8_bom = false
crl = true
block_comments = false
"#;
        let config: Config = toml::from_str(toml_str).unwrap();

        assert_eq!(config.verbose, Some(3));
        assert_eq!(config.line_length, Some(120));
        assert_eq!(config.output.as_deref(), Some("vs7"));
        assert_eq!(config.recursive, Some(true));
        assert_eq!(config.quiet, Some(true));
        assert_eq!(config.repository.as_deref(), Some("/path/to/repo"));
        assert_eq!(config.root.as_deref(), Some("src"));
        assert_eq!(config.include_order.as_deref(), Some("standardcfirst"));

        let filters = config.filters.unwrap();
        assert_eq!(filters.add, vec!["-build/include_alpha", "+build/include"]);

        let ext = config.extensions.unwrap();
        assert_eq!(ext.headers.unwrap(), vec!["h", "hpp"]);
        assert_eq!(ext.sources.unwrap(), vec!["c", "cpp"]);

        let exclude = config.exclude.unwrap();
        assert_eq!(exclude.files, vec!["build/**", "third_party/**"]);

        let fix = config.fix.unwrap();
        assert_eq!(fix.trailing_whitespace, Some(true));
        assert_eq!(fix.utf8_bom, Some(false));
        assert_eq!(fix.crl, Some(true));
        assert_eq!(fix.block_comments, Some(false));
    }

    #[test]
    fn test_defaults() {
        let config = Config::default();
        assert_eq!(config.effective_verbose(), 1);
        assert_eq!(config.effective_line_length(), 80);
        assert_eq!(config.effective_output(), "emacs");
    }

    #[test]
    fn test_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.effective_verbose(), 1);
        assert_eq!(config.effective_line_length(), 80);
        assert_eq!(config.effective_output(), "emacs");
    }

    #[test]
    fn test_fix_config_defaults() {
        let fix = FixConfig::default();
        assert_eq!(fix.trailing_whitespace, Some(true));
        assert_eq!(fix.utf8_bom, Some(true));
        assert_eq!(fix.crl, Some(true));
        assert_eq!(fix.block_comments, Some(true));
    }

    #[test]
    fn test_extension_config_defaults() {
        let ext = ExtensionConfig::default();
        let headers = ext.headers.unwrap();
        assert_eq!(headers, vec!["h", "hh", "hpp", "hxx", "h++", "cuh"]);
        let sources = ext.sources.unwrap();
        assert_eq!(sources, vec!["c", "cc", "cpp", "cxx", "c++", "cu"]);
    }
}
