use crate::cli::output::{print_json, print_skills_table};
use crate::model::SkillJsonView;
use crate::services::SkillEngine;
use anyhow::Result;

use super::shared::load_index_or_report;

/// Handle `skillscripts list`.
pub(crate) fn run_list(engine: &SkillEngine, json: bool) -> Result<()> {
    let index = load_index_or_report(engine)?;

    if json {
        let skills: Vec<_> = index.skills.iter().map(SkillJsonView::from).collect();
        print_json(&skills)?;
        return Ok(());
    }

    let skill_refs: Vec<_> = index.skills.iter().collect();
    print_skills_table(&skill_refs);
    println!("Total {} skills", skill_refs.len());

    Ok(())
}
