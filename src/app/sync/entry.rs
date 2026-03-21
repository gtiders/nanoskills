use crate::domain::{ParseError, Skill};
use crate::infra::HeaderParser;
use std::path::Path;

pub(super) enum SyncEntry {
    Skill(Skill),
    Error(ParseError),
    Skipped,
}

pub(super) fn parse_skill_file(file_path: &str, strict: bool) -> SyncEntry {
    match HeaderParser::parse_file(Path::new(file_path)) {
        Ok(Some(header)) => SyncEntry::Skill(Skill::from((header, file_path.to_string()))),
        Ok(None) => SyncEntry::Skipped,
        Err(error) if strict => {
            SyncEntry::Error(ParseError::new(file_path.to_string(), error.to_string()))
        }
        Err(_) => SyncEntry::Skipped,
    }
}
