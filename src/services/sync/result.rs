use crate::model::ParseError;

/// Summary returned by `sync`.
pub(crate) struct SyncResult {
    pub(crate) total_files: usize,
    pub(crate) skills_count: usize,
    pub(crate) elapsed_ms: u128,
    pub(crate) errors: Vec<ParseError>,
}
