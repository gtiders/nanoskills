use crate::app::SkillEngine;
use crate::presentation::output::{
    print_detailed_table, print_json_highlighted, print_skills_table,
};
use anyhow::Result;
use rust_i18n::t;

use super::shared::load_index_or_report;

/// Handle `nanoskills list`.
pub(crate) fn run_list(engine: &SkillEngine, json: bool, detailed: bool) -> Result<()> {
    let Some(index) = load_index_or_report(engine)? else {
        return Ok(());
    };

    if json {
        print_json_highlighted(&index.skills, "JSON")?;
        return Ok(());
    }

    let skill_refs: Vec<_> = index.skills.iter().collect();
    if detailed {
        print_detailed_table(&skill_refs);
    } else {
        print_skills_table(&skill_refs);
        println!("{}", t!("cli.total_skills", count = skill_refs.len()));
    }

    Ok(())
}
