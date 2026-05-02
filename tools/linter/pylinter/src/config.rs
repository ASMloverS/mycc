use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ── Enums ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisLevel {
    None,
    #[default]
    Basic,
    Strict,
    Deep,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LineEnding {
    #[default]
    Lf,
    Crlf,
    Native,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportSorting {
    #[default]
    Pep8,
    #[serde(rename = "none")]
    Disabled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryOpBreak {
    #[default]
    Before,
    After,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentStyle {
    #[default]
    #[serde(rename = "hash")]
    Hash,
    Preserve,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamingStyle {
    #[default]
    SnakeCase,
    UpperSnakeCase,
    PascalCase,
    CamelCase,
}

impl NamingStyle {
    pub fn as_str(&self) -> &'static str {
        match self {
            NamingStyle::SnakeCase => "snake_case",
            NamingStyle::UpperSnakeCase => "upper_snake_case",
            NamingStyle::PascalCase => "pascal_case",
            NamingStyle::CamelCase => "camel_case",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrailingWhitespace {
    #[default]
    #[serde(rename = "strip")]
    Strip,
    #[serde(rename = "preserve")]
    Preserve,
}

// ── Config structs ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FormatConfig {
    pub indent_width: usize,
    pub use_tabs: bool,
    pub column_limit: usize,
    pub line_ending: LineEnding,
    pub encoding: String,
    pub trailing_whitespace: TrailingWhitespace,
    pub blank_lines_before_class: usize,
    pub blank_lines_before_function: usize,
    pub blank_lines_inside_class: usize,
    pub max_consecutive_blank_lines: usize,
    pub import_sorting: ImportSorting,
    pub binary_op_line_break: BinaryOpBreak,
    pub comment_style: CommentStyle,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_width: 4,
            use_tabs: false,
            column_limit: 120,
            line_ending: LineEnding::Lf,
            encoding: "utf-8".into(),
            trailing_whitespace: TrailingWhitespace::Strip,
            blank_lines_before_class: 2,
            blank_lines_before_function: 2,
            blank_lines_inside_class: 1,
            max_consecutive_blank_lines: 2,
            import_sorting: ImportSorting::Pep8,
            binary_op_line_break: BinaryOpBreak::Before,
            comment_style: CommentStyle::Hash,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NamingConfig {
    pub function: NamingStyle,
    pub class: NamingStyle,
    pub constant: NamingStyle,
    pub variable: NamingStyle,
    pub module: NamingStyle,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            function: NamingStyle::SnakeCase,
            class: NamingStyle::PascalCase,
            constant: NamingStyle::UpperSnakeCase,
            variable: NamingStyle::SnakeCase,
            module: NamingStyle::SnakeCase,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ComplexityConfig {
    pub max_function_lines: usize,
    pub max_class_lines: usize,
    pub max_file_lines: usize,
    pub max_nesting_depth: usize,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_function_lines: 50,
            max_class_lines: 300,
            max_file_lines: 1000,
            max_nesting_depth: 4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MagicNumberConfig {
    pub enabled: bool,
    pub allowed: Vec<i64>,
}

impl Default for MagicNumberConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed: vec![0, 1, -1, 2],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UnusedImportConfig {
    pub enabled: bool,
}

impl Default for UnusedImportConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProhibitedConfig {
    pub use_default: bool,
    pub extra: Vec<String>,
    pub remove: Vec<String>,
}

impl Default for ProhibitedConfig {
    fn default() -> Self {
        Self {
            use_default: true,
            extra: vec![],
            remove: vec![],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DocstringConfig {
    pub module: bool,
    pub class: bool,
    pub function: bool,
}

impl Default for DocstringConfig {
    fn default() -> Self {
        Self {
            module: true,
            class: true,
            function: true,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CheckConfig {
    pub naming: NamingConfig,
    pub complexity: ComplexityConfig,
    pub magic_number: MagicNumberConfig,
    pub unused_import: UnusedImportConfig,
    pub prohibited: ProhibitedConfig,
    pub docstring: DocstringConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AnalysisConfig {
    pub level: AnalysisLevel,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            level: AnalysisLevel::Basic,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}

// ── Config loading ─────────────────────────────────────────────────────────

const CONFIG_FILENAME: &str = ".pylinter.yaml";

pub fn find_config() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("cannot determine working directory: {e}"))?;
    let config_name = PathBuf::from(CONFIG_FILENAME);

    let mut dir: &std::path::Path = &cwd;
    loop {
        let candidate = dir.join(&config_name);
        if candidate.exists() {
            return Ok(Some(candidate));
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => break,
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let exe_config = exe_dir.join(&config_name);
            if exe_config.exists() {
                return Ok(Some(exe_config));
            }
        }
    }

    Ok(None)
}

pub fn load_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    let Some(p) = (match path {
        Some(p) => {
            if !p.exists() {
                return Err(format!("config file not found: {}", p.display()).into());
            }
            Some(p.clone())
        }
        None => find_config()?,
    }) else {
        return Ok(Config::default());
    };

    let content = std::fs::read_to_string(&p)
        .map_err(|e| format!("cannot read config {}: {e}", p.display()))?;
    Ok(serde_yaml::from_str(&content)
        .map_err(|e| format!("invalid config {}: {e}", p.display()))?)
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config, parsed);
    }

    #[test]
    fn partial_yaml_uses_defaults() {
        let yaml = "format:\n  indent_width: 2\n";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.format.indent_width, 2);
        // Everything else should be defaults
        assert_eq!(config.format.column_limit, 120);
        assert_eq!(config.format.line_ending, LineEnding::Lf);
        assert_eq!(config.format.encoding, "utf-8");
        assert_eq!(config.format.trailing_whitespace, TrailingWhitespace::Strip);
        assert_eq!(config.check.naming.function, NamingStyle::SnakeCase);
        assert_eq!(config.check.complexity.max_function_lines, 50);
        assert_eq!(config.analysis.level, AnalysisLevel::Basic);
    }

    #[test]
    fn all_naming_styles_str() {
        assert_eq!(NamingStyle::SnakeCase.as_str(), "snake_case");
        assert_eq!(NamingStyle::UpperSnakeCase.as_str(), "upper_snake_case");
        assert_eq!(NamingStyle::PascalCase.as_str(), "pascal_case");
        assert_eq!(NamingStyle::CamelCase.as_str(), "camel_case");
    }

    #[test]
    fn shipped_yaml_deserializes() {
        let yaml = include_str!("../.pylinter.yaml");
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn load_config_with_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test-config.yaml");
        std::fs::write(&file_path, "format:\n  indent_width: 2\n").unwrap();
        let config = load_config(Some(&file_path)).unwrap();
        assert_eq!(config.format.indent_width, 2);
        assert_eq!(config.format.column_limit, 120);
    }

    #[test]
    fn load_config_missing_file_returns_error() {
        let path = PathBuf::from("/nonexistent/.pylinter.yaml");
        let result = load_config(Some(&path));
        assert!(result.is_err());
    }
}
