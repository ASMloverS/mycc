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

    pub fn display_path(&self) -> String {
        self.path.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string_and_line_access() {
        let src = SourceFile::from_string("line1\nline2\nline3\n", PathBuf::from("test.py"));
        assert_eq!(src.line_count(), 3);
        assert_eq!(src.lines(), vec!["line1", "line2", "line3"]);
        assert!(!src.is_modified());
    }

    #[test]
    fn modification_tracking() {
        let mut src = SourceFile::from_string("hello", PathBuf::from("test.py"));
        assert!(!src.is_modified());
        src.content = "world".into();
        assert!(src.is_modified());
    }
}
