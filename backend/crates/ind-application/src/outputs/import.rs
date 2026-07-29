use ind_domain::{ImportJob, ImportJobItem};

#[derive(Debug, Clone)]
pub struct ImportStatusOutput {
    pub job: ImportJob,
    pub items: Vec<ImportJobItem>,
}
