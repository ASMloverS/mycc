use crate::blame::{BlameEntry, LineSpec, VcsKind};
use crate::util::AppError;

pub mod detect;
pub mod git;
pub mod svn;

pub struct FileDiff {
    pub file: String,
    pub hunks: Vec<Hunk>,
}

pub struct Hunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub added_lines: Vec<usize>,
}

pub trait VcsBackend {
    fn name(&self) -> &str;
    fn kind(&self) -> VcsKind;
    fn blame_file(&self, file: &str, lines: &LineSpec) -> Result<Vec<BlameEntry>, AppError>;
    fn diff_revisions(&self, base: &str, head: &str) -> Result<Vec<FileDiff>, AppError>;
}
