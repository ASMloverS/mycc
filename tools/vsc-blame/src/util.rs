use std::fmt;
use std::sync::LazyLock;

static REF_PATTERN: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[A-Za-z0-9._/~^@:-]+$").unwrap()
});

#[derive(Debug)]
pub struct AppError {
    pub message: String,
    pub code: i32,
}

impl AppError {
    pub fn new(msg: impl Into<String>, code: i32) -> Self {
        Self {
            message: msg.into(),
            code,
        }
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(msg, 1)
    }
    pub fn usage(msg: impl Into<String>) -> Self {
        Self::new(msg, 2)
    }
    pub fn vcs_not_found(msg: impl Into<String>) -> Self {
        Self::new(msg, 3)
    }
    pub fn file_not_found(msg: impl Into<String>) -> Self {
        Self::new(msg, 4)
    }
    pub fn empty(msg: impl Into<String>) -> Self {
        Self::new(msg, 5)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ERROR] {}", self.message)
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        Self::error(e.to_string())
    }
}

pub fn validate_ref(val: &str) -> Result<(), AppError> {
    if REF_PATTERN.is_match(val) {
        Ok(())
    } else {
        Err(AppError::usage(format!("invalid ref/revision value: {}", val)))
    }
}

pub fn warn(msg: &str, quiet: bool) {
    if !quiet {
        eprintln!("[WARN] {}", msg);
    }
}

pub fn short_commit_id(id: &str) -> &str {
    if id.len() >= 7 { &id[..7] } else { id }
}

pub trait IoResultExt<T> {
    fn map_io_err(self) -> Result<T, AppError>;
}

impl<T> IoResultExt<T> for Result<T, std::io::Error> {
    fn map_io_err(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::error(e.to_string()))
    }
}

pub fn read_input(
    text: Option<&str>,
    file: Option<&str>,
    use_stdin: bool,
    label: &str,
) -> Result<String, AppError> {
    if use_stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| AppError::error(format!("failed to read stdin: {}", e)))?;
        return Ok(buf);
    }
    if let Some(p) = file {
        return std::fs::read_to_string(p)
            .map_err(|e| AppError::file_not_found(format!("cannot read {}: {}", p, e)));
    }
    if let Some(t) = text {
        return Ok(t.to_string());
    }
    Err(AppError::usage(format!("no input: specify {}, -f <file>, or --stdin", label)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ref_valid() {
        assert!(validate_ref("HEAD").is_ok());
        assert!(validate_ref("main").is_ok());
        assert!(validate_ref("HEAD~3").is_ok());
        assert!(validate_ref("abc1234").is_ok());
        assert!(validate_ref("r42").is_ok());
        assert!(validate_ref("tags/v1.0").is_ok());
    }

    #[test]
    fn test_validate_ref_invalid() {
        assert!(validate_ref("foo; rm -rf /").is_err());
        assert!(validate_ref("$(whoami)").is_err());
        assert!(validate_ref("").is_err());
    }
}
