use crate::domain::{Index, ParseError};
use rayon::prelude::*;

use super::entry::{SyncEntry, parse_skill_file};
use super::tool_name_resolver::finalize_tool_names;

pub(crate) fn build_index(files: &[String], strict: bool) -> (Index, Vec<ParseError>) {
    let results: Vec<_> = files
        .par_iter()
        .map(|file_path| parse_skill_file(file_path, strict))
        .collect();

    let mut skills = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            SyncEntry::Skill(skill) => skills.push(skill),
            SyncEntry::Error(error) => errors.push(error),
            SyncEntry::Skipped => {}
        }
    }

    errors.extend(finalize_tool_names(&mut skills, strict));
    if strict {
        skills.retain(|skill| skill.tool_name.is_some());
    }

    skills.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.path.cmp(&right.path))
    });

    let mut index = Index::new();
    index.skills = skills;

    (index, errors)
}
