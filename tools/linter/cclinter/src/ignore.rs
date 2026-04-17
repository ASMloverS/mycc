use std::path::Path;

pub struct IgnoreMatcher {
    _patterns: Vec<globset::GlobMatcher>,
}

impl IgnoreMatcher {
    pub fn from_file(_path: &Path) -> Self {
        Self { _patterns: vec![] }
    }

    pub fn is_ignored(&self, _path: &Path) -> bool {
        false
    }
}
