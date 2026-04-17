use crate::common::diag::Severity;

pub trait Rule {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn severity(&self) -> Severity;
}
