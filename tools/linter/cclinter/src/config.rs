use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisLevel {
    #[default]
    None,
    Basic,
    Strict,
    Deep,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointerAlignment {
    #[default]
    Left,
    Right,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BraceStyle {
    #[default]
    Attach,
    Breakout,
    #[serde(rename = "attach_breakout")]
    AttachBreakout,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncludeSorting {
    #[default]
    Google,
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentStyle {
    #[default]
    #[serde(rename = "double_slash")]
    DoubleSlash,
    Preserve,
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
pub enum NamingStyle {
    #[default]
    SnakeCase,
    UpperSnakeCase,
    PascalCase,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncludeGuardStyle {
    #[default]
    #[serde(rename = "pragma_once")]
    PragmaOnce,
    Ifndef,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FormatConfig {
    pub column_limit: usize,
    pub indent_width: usize,
    pub use_tabs: bool,
    pub pointer_alignment: PointerAlignment,
    pub brace_style: BraceStyle,
    pub switch_case_indent: bool,
    pub blank_lines_before_function: usize,
    pub blank_lines_after_include: usize,
    pub max_consecutive_blank_lines: usize,
    pub space_before_paren: bool,
    pub spaces_around_operators: bool,
    pub include_sorting: IncludeSorting,
    pub comment_style: CommentStyle,
    pub line_ending: LineEnding,
    pub encoding: String,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            column_limit: 120,
            indent_width: 2,
            use_tabs: false,
            pointer_alignment: PointerAlignment::Left,
            brace_style: BraceStyle::Attach,
            switch_case_indent: true,
            blank_lines_before_function: 1,
            blank_lines_after_include: 1,
            max_consecutive_blank_lines: 2,
            space_before_paren: false,
            spaces_around_operators: true,
            include_sorting: IncludeSorting::Google,
            comment_style: CommentStyle::DoubleSlash,
            line_ending: LineEnding::Lf,
            encoding: "utf-8".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NamingConfig {
    pub function: NamingStyle,
    pub variable: NamingStyle,
    pub constant: NamingStyle,
    pub r#type: NamingStyle,
    pub r#macro: NamingStyle,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            function: NamingStyle::SnakeCase,
            variable: NamingStyle::SnakeCase,
            constant: NamingStyle::UpperSnakeCase,
            r#type: NamingStyle::PascalCase,
            r#macro: NamingStyle::UpperSnakeCase,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ComplexityConfig {
    pub max_function_lines: usize,
    pub max_file_lines: usize,
    pub max_nesting_depth: usize,
}

impl Default for ComplexityConfig {
    fn default() -> Self {
        Self {
            max_function_lines: 100,
            max_file_lines: 2000,
            max_nesting_depth: 5,
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
pub struct IncludeGuardConfig {
    pub style: IncludeGuardStyle,
}

impl Default for IncludeGuardConfig {
    fn default() -> Self {
        Self {
            style: IncludeGuardStyle::PragmaOnce,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProhibitedFunctionsConfig {
    pub use_default: bool,
    pub extra: Vec<String>,
    pub remove: Vec<String>,
}

impl Default for ProhibitedFunctionsConfig {
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
pub struct CheckConfig {
    pub naming: NamingConfig,
    pub complexity: ComplexityConfig,
    pub magic_number: MagicNumberConfig,
    pub include_guard: IncludeGuardConfig,
    pub prohibited_functions: ProhibitedFunctionsConfig,
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            naming: NamingConfig::default(),
            complexity: ComplexityConfig::default(),
            magic_number: MagicNumberConfig::default(),
            include_guard: IncludeGuardConfig::default(),
            prohibited_functions: ProhibitedFunctionsConfig::default(),
        }
    }
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: FormatConfig::default(),
            check: CheckConfig::default(),
            analysis: AnalysisConfig::default(),
        }
    }
}

pub fn load_config(path: Option<&PathBuf>) -> Config {
    match path {
        Some(p) if p.exists() => {
            let content = std::fs::read_to_string(p).unwrap_or_default();
            serde_yaml::from_str(&content).unwrap_or_default()
        }
        _ => Config::default(),
    }
}

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
        let yaml = "format:\n  column_limit: 80\n";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.format.column_limit, 80);
        assert_eq!(config.format.indent_width, 2);
    }
}
