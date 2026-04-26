use crate::common::diag::{Diagnostic, Severity};
use crate::common::source::SourceFile;

pub trait Rule {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn severity(&self) -> Severity;
    fn check(&self, source: &SourceFile) -> Vec<Diagnostic>;
}
