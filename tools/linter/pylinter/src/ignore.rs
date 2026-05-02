pub struct IgnoreMatcher;

impl IgnoreMatcher {
    pub fn from_patterns(_patterns: &[String]) -> Self {
        Self
    }

    pub fn is_ignored(&self, _path: &std::path::Path) -> bool {
        false
    }
}
