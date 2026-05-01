# Task 02: Config system — config.rs + .pylinter.yaml

> Status: ⬜ Not started
> Deps: Task 01
> Output: config load/serialize/deserialize/defaults, unit tests pass

## Goal

Full config system. Match cclinter `config.rs` style.

## Reference

- `cclinter/src/config.rs` — enum defs, serde derives, `load_config` logic
- `cclinter/.cclinter.yaml` — default config format

## Steps

### 1. Enums

```rust
// Follow cclinter pattern for AnalysisLevel, BraceStyle, etc.

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
```

### 2. Config structs

```rust
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

// Default impl: indent_width=4, column_limit=120, line_ending=Lf, encoding="utf-8"

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NamingConfig {
    pub function: NamingStyle,
    pub class: NamingStyle,
    pub constant: NamingStyle,
    pub variable: NamingStyle,
    pub module: NamingStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ComplexityConfig { /* per design doc */ }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct MagicNumberConfig { /* per design doc */ }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct UnusedImportConfig { pub enabled: bool }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ProhibitedConfig { pub use_default: bool, pub extra: Vec<String>, pub remove: Vec<String> }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DocstringConfig { pub module: bool, pub class: bool, pub function: bool }

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CheckConfig { /* all check sub-configs above */ }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AnalysisConfig { pub level: AnalysisLevel }

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}
```

### 3. Config loading

Match cclinter `load_config` / `find_config`:

```rust
const CONFIG_FILENAME: &str = ".pylinter.yaml";

pub fn find_config() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    // Same as cclinter: cwd -> parent walk -> exe dir
}

pub fn load_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    // Same as cclinter: given path or auto-discover or defaults
}
```

### 4. Create .pylinter.yaml

Write design doc Section 4 YAML → `tools/linter/pylinter/.pylinter.yaml`.

## Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn default_config_roundtrip() {
        // serialize → deserialize, assert equal
    }

    #[test]
    fn partial_yaml_uses_defaults() {
        // override only indent_width, verify rest are defaults
    }

    #[test]
    fn all_naming_styles_str() {
        // verify as_str() for each NamingStyle variant
    }
}
```

## Verify

```bash
cargo test -- config
```
