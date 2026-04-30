### Task 13: YAML Config Loading + Directory Lookup

**Files:**
- Modify: `tools/linter/cclinter/src/config.rs`
- Modify: `tools/linter/cclinter/src/cli.rs`
- Test: `tools/linter/cclinter/tests/` (config tests)

- [x] **Step 1: Write failing tests**

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

- [x] **Step 2: Run tests to verify failure**

Run: `cargo test --test config_tests`
Expected: FAIL — `find_config` may not exist yet.

- [x] **Step 3: Expand `src/config.rs` with full config struct**

All config structs use `#[serde(default)]` with non-Optional fields and explicit `Default` impls.

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Enums (all derive Clone, Debug, Default, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize):
//   AnalysisLevel: None, Basic(default), Strict, Deep
//   PointerAlignment: Left(default), Right
//   BraceStyle: Attach(default), Breakout, AttachBreakout
//   IncludeSorting: Google(default), Disabled
//   CommentStyle: DoubleSlash(default), Preserve
//   LineEnding: Lf(default), Crlf, Native
//   NamingStyle: SnakeCase(default), UpperSnakeCase, PascalCase
//   IncludeGuardStyle: PragmaOnce(default), Ifndef

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    pub format: FormatConfig,
    pub check: CheckConfig,
    pub analysis: AnalysisConfig,
}

// FormatConfig: all non-Optional fields with defaults (see design doc)
// CheckConfig: naming, complexity, magic_number, unused, include_guard, prohibited_functions
// AnalysisConfig: level (AnalysisLevel, default Basic)

pub fn find_config() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
    // 1. CWD .cclinter.yaml
    // 2. Walk parent dirs
    // 3. Tool binary dir (current_exe().parent())
}

pub fn load_config(path: Option<&PathBuf>) -> Result<Config, Box<dyn std::error::Error>> {
    // --config → fail if missing
    // No --config → find_config() → parse or default
}
```

Key: `find_config()` returns `Option<PathBuf>`, `load_config()` returns `Config`. Config structs have `PartialEq` for testing. `serde(rename_all = "snake_case")` and special renames (`attach_breakout`, `double_slash`, `none`, `pragma_once`).

- [x] **Step 4: Update `src/cli.rs` to load config**

Update `run()`:

```rust
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config(args.config.as_ref())?;
    if let Some(ref level) = args.analysis_level {
        config.analysis.level = level.clone();
    }
    // ... rest of pipeline
}
```

Note: `load_config` returns `Result<Config, ...>`. `--analysis-level` overrides `config.analysis.level`.

- [x] **Step 5: Create default `.cclinter.yaml` in project root**

Copy the YAML config from the design doc into `tools/linter/cclinter/.cclinter.yaml`.

- [x] **Step 6: Run tests**

Run: `cargo test --test config_tests`
Expected: All PASS.

- [x] **Step 7: Commit**

```bash
git add tools/linter/cclinter/
git commit -m "✨ feat(cclinter): full YAML config with directory lookup"
```
