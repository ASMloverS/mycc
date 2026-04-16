### Task 13: YAML Config Loading + Directory Lookup

**Files:**
- Modify: `tools/linter/cclinter/src/config.rs`
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tools/linter/cclinter/tests/` (config tests)

- [ ] **Step 1: Write failing tests**

Create `tests/config_tests.rs`:

```rust
use cclinter::config::{load_config, find_config, Config};
use std::path::PathBuf;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.format.column_limit, Some(120));
    assert_eq!(config.format.indent_width, Some(2));
}

#[test]
fn test_parse_yaml_config() {
    let yaml = r#"
format:
  column_limit: 100
  indent_width: 4
  pointer_alignment: right
  brace_style: breakout
"#;
    let config: Config = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.format.column_limit, Some(100));
    assert_eq!(config.format.indent_width, Some(4));
}

#[test]
fn test_find_config_in_tool_dir() {
    let exe_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    let result = find_config(None, &exe_dir);
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test --test config_tests`
Expected: FAIL — `find_config` may not exist yet.

- [ ] **Step 3: Expand `src/config.rs` with full config struct**

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub format: FormatConfig,
    #[serde(default)]
    pub check: CheckConfig,
    #[serde(default)]
    pub analysis: AnalysisConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FormatConfig {
    pub column_limit: Option<usize>,
    pub indent_width: Option<usize>,
    pub use_tabs: Option<bool>,
    pub pointer_alignment: Option<String>,
    pub brace_style: Option<String>,
    pub switch_case_indent: Option<bool>,
    pub blank_lines_before_function: Option<usize>,
    pub blank_lines_after_include: Option<usize>,
    pub max_consecutive_blank_lines: Option<usize>,
    pub space_before_paren: Option<bool>,
    pub spaces_around_operators: Option<bool>,
    pub include_sorting: Option<String>,
    pub comment_style: Option<String>,
    pub line_ending: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckConfig {
    pub naming: Option<NamingConfig>,
    pub complexity: Option<ComplexityConfig>,
    pub magic_number: Option<MagicNumberConfig>,
    pub include_guard: Option<IncludeGuardConfig>,
    pub prohibited_functions: Option<ProhibitedFunctionsConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NamingConfig {
    pub function: Option<String>,
    pub variable: Option<String>,
    pub constant: Option<String>,
    pub r#type: Option<String>,
    pub macro: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ComplexityConfig {
    pub max_function_lines: Option<usize>,
    pub max_file_lines: Option<usize>,
    pub max_nesting_depth: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MagicNumberConfig {
    pub enabled: Option<bool>,
    pub allowed: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IncludeGuardConfig {
    pub style: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProhibitedFunctionsConfig {
    pub use_default: Option<bool>,
    pub extra: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AnalysisConfig {
    pub level: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: FormatConfig {
                column_limit: Some(120),
                indent_width: Some(2),
                use_tabs: Some(false),
                pointer_alignment: Some("left".into()),
                brace_style: Some("attach".into()),
                switch_case_indent: Some(true),
                blank_lines_before_function: Some(1),
                blank_lines_after_include: Some(1),
                max_consecutive_blank_lines: Some(2),
                space_before_paren: Some(false),
                spaces_around_operators: Some(true),
                include_sorting: Some("google".into()),
                comment_style: Some("double_slash".into()),
                line_ending: Some("lf".into()),
                encoding: Some("utf-8".into()),
            },
            check: CheckConfig {
                naming: Some(NamingConfig {
                    function: Some("snake_case".into()),
                    variable: Some("snake_case".into()),
                    constant: Some("upper_snake_case".into()),
                    r#type: Some("pascal_case".into()),
                    macro: Some("upper_snake_case".into()),
                }),
                complexity: Some(ComplexityConfig {
                    max_function_lines: Some(100),
                    max_file_lines: Some(2000),
                    max_nesting_depth: Some(5),
                }),
                magic_number: Some(MagicNumberConfig {
                    enabled: Some(true),
                    allowed: Some(vec![0, 1, -1, 2]),
                }),
                include_guard: Some(IncludeGuardConfig {
                    style: Some("pragma_once".into()),
                }),
                prohibited_functions: Some(ProhibitedFunctionsConfig {
                    use_default: Some(true),
                    extra: Some(vec![]),
                    remove: Some(vec![]),
                }),
            },
            analysis: AnalysisConfig {
                level: Some("basic".into()),
            },
        }
    }
}

impl Default for FormatConfig {
    fn default() -> Self {
        Config::default().format
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Config::default().check
    }
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Config::default().analysis
    }
}

pub fn find_config(cli_path: Option<&PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    const CONFIG_NAME: &str = ".cclinter.yaml";

    if let Some(p) = cli_path {
        let content = std::fs::read_to_string(p)?;
        let config: Config = serde_yaml::from_str(&content)?;
        return Ok(config);
    }

    let config_path = search_config(CONFIG_NAME);
    match config_path {
        Some(path) => {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = serde_yaml::from_str(&content)?;
            Ok(config)
        }
        None => Ok(Config::default()),
    }
}

fn search_config(name: &str) -> Option<PathBuf> {
    // 1. CWD
    let cwd = std::env::current_dir().ok()?;
    if let Some(p) = walk_ancestors(&cwd, name) {
        return Some(p);
    }
    // 2. Tool binary directory
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

fn walk_ancestors(start: &std::path::Path, name: &str) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
}
```

- [ ] **Step 4: Update `src/cli.rs` to load config**

Update `run()`:

```rust
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = crate::config::find_config(args.config.as_ref())?;
    println!("cclinter: loaded config (column_limit={:?})", config.format.column_limit);
    Ok(())
}
```

- [ ] **Step 5: Create default `.cclinter.yaml` in project root**

Copy the YAML config from the design doc into `tools/linter/cclinter/.cclinter.yaml`.

- [ ] **Step 6: Run tests**

Run: `cargo test --test config_tests`
Expected: All PASS.

- [ ] **Step 7: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): full YAML config with directory lookup"
```
