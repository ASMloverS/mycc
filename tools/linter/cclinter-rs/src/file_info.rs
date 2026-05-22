use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub filename: String,
    pub basename: String,
    pub stem: String,
    pub extension: String,
    pub is_header: bool,
    pub repository_name: String,
    pub file_extension: String,
}

impl FileInfo {
    pub fn new(filename: &str, repository: &str, header_extensions: &HashSet<String>) -> Self {
        let path = Path::new(filename);
        let basename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let stem = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let is_header = header_extensions.contains(&extension);
        let repository_name = if !repository.is_empty()
            && filename.starts_with(repository)
            && filename.len() > repository.len()
        {
            filename[repository.len()..]
                .strip_prefix('/')
                .unwrap_or(&filename[repository.len()..])
                .to_string()
        } else {
            filename.to_string()
        };
        let file_extension = extension.clone();
        FileInfo {
            filename: filename.to_string(),
            basename,
            stem,
            extension,
            is_header,
            repository_name,
            file_extension,
        }
    }

    pub fn header_guard(&self, root: &str) -> String {
        let path = if !root.is_empty()
            && self.filename.starts_with(root)
            && self.filename.len() > root.len()
        {
            &self.filename[root.len()..]
        } else {
            &self.filename
        };
        let path = path.strip_prefix('/').unwrap_or(path);
        let without_ext = if !self.extension.is_empty() {
            &path[..path.len() - self.extension.len() - 1]
        } else {
            path
        };
        without_ext
            .to_uppercase()
            .replace(['/', '.'], "_")
            + "_H_"
    }

    pub fn repository_name(&self) -> &str {
        &self.repository_name
    }

    pub fn is_c_file(&self) -> bool {
        self.extension == "c"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_ext() -> HashSet<String> {
        [".h", ".hpp", ".hh", ".hxx", ".h++"]
            .iter()
            .map(|s| s[1..].to_string())
            .collect()
    }

    #[test]
    fn test_basic_file_info() {
        let exts = header_ext();
        let fi = FileInfo::new("src/foo/bar.cc", "", &exts);
        assert_eq!(fi.basename, "bar.cc");
        assert_eq!(fi.stem, "bar");
        assert_eq!(fi.extension, "cc");
        assert!(!fi.is_header);
    }

    #[test]
    fn test_header_file() {
        let exts = header_ext();
        let fi = FileInfo::new("src/foo/bar.h", "", &exts);
        assert!(fi.is_header);
    }

    #[test]
    fn test_header_guard() {
        let exts = header_ext();
        let fi = FileInfo::new("src/foo/bar.h", "", &exts);
        assert_eq!(fi.header_guard(""), "SRC_FOO_BAR_H_");
    }

    #[test]
    fn test_header_guard_with_root() {
        let exts = header_ext();
        let fi = FileInfo::new("src/foo/bar.h", "", &exts);
        assert_eq!(fi.header_guard("src"), "FOO_BAR_H_");
    }

    #[test]
    fn test_repository_name() {
        let exts = header_ext();
        let fi = FileInfo::new("repo/src/foo.cc", "repo", &exts);
        assert_eq!(fi.repository_name(), "src/foo.cc");
    }

    #[test]
    fn test_is_c_file() {
        let exts = header_ext();
        let fi_c = FileInfo::new("test.c", "", &exts);
        let fi_cc = FileInfo::new("test.cc", "", &exts);
        assert!(fi_c.is_c_file());
        assert!(!fi_cc.is_c_file());
    }
}
