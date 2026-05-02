#[derive(Debug, Clone, Default)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Clone)]
pub struct FormatConfig {
    pub indent_width: usize,
    pub use_tabs: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_width: 4,
            use_tabs: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CheckConfig;

#[derive(Debug, Clone, Default)]
pub struct AnalysisConfig;

#[derive(Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum AnalysisLevel {
    None,
    #[default]
    Basic,
    Strict,
    Deep,
}
