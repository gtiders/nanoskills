use crate::app::SkillEngine;
use crate::domain::OpenAITool;
use crate::presentation::output::{print_json_highlighted, print_skills_table};
use anyhow::Result;
use rust_i18n::t;

use super::shared::load_index_or_report;

/// Handle `nanoskills search`.
pub(crate) fn run_search(
    engine: &SkillEngine,
    query: &str,
    json: bool,
    limit: Option<usize>,
) -> Result<()> {
    let Some(index) = load_index_or_report(engine)? else {
        return Ok(());
    };

    let search_limit = engine.resolve_search_limit(&std::env::current_dir()?, limit)?;
    let results = engine.search(&index.skills, query);
    let limited_results: Vec<_> = results.into_iter().take(search_limit).collect();

    if json {
        let tools: Vec<_> = limited_results
            .iter()
            .map(|(skill, _)| OpenAITool::from(*skill))
            .collect();
        print_json_highlighted(&tools, "JSON")?;
        return Ok(());
    }

    if limited_results.is_empty() {
        println!("{}", t!("cli.search_not_found"));
        return Ok(());
    }

    println!(
        "{}",
        t!(
            "cli.search_found",
            count = limited_results.len(),
            limit = search_limit
        )
    );

    let skill_refs: Vec<_> = limited_results.iter().map(|(skill, _)| *skill).collect();
    print_skills_table(&skill_refs);
    println!("{}", t!("cli.search_hint"));

    Ok(())
}
