use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
    pub original: String,
}

impl SourceFile {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            original: content.clone(),
            content,
        })
    }

    pub fn from_string(content: &str, path: PathBuf) -> Self {
        Self {
            path,
            original: content.to_string(),
            content: content.to_string(),
        }
    }

    pub fn lines(&self) -> Vec<&str> {
        self.content.lines().collect()
    }

    pub fn line_count(&self) -> usize {
        self.content.lines().count()
    }

    pub fn write(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.write_to(&self.path)
    }

    pub fn write_to(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &self.content)?;
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.content != self.original
    }
}

pub fn mask_string_literals(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut in_string = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && in_string {
            result.push('_');
            if chars.peek().is_some() {
                result.push('_');
                chars.next();
            }
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            result.push('"');
        } else if in_string {
            result.push('_');
        } else {
            result.push(c);
        }
    }
    result
}

pub fn strip_line_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut chars = line.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '"' && (i == 0 || line.as_bytes()[i - 1] != b'\\') {
            in_string = !in_string;
        } else if c == '/' && !in_string {
            if chars.peek().map(|(_, c)| *c) == Some('/') {
                return &line[..i];
            }
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_and_line_access() {
        let src = SourceFile::from_string("line1\nline2\nline3\n", PathBuf::from("test.c"));
        assert_eq!(src.line_count(), 3);
        assert_eq!(src.lines(), vec!["line1", "line2", "line3"]);
        assert!(!src.is_modified());
    }

    #[test]
    fn modification_tracking() {
        let mut src = SourceFile::from_string("hello", PathBuf::from("test.c"));
        assert!(!src.is_modified());
        src.content = "world".into();
        assert!(src.is_modified());
    }
}
