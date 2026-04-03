use crate::cli::output::print_json;
use crate::model::SkillJsonView;
use crate::services::SkillEngine;
use anyhow::Result;

use super::shared::load_index_or_report;

/// Handle `nanoskills search`.
pub(crate) fn run_search(
    engine: &SkillEngine,
    query: &str,
    limit: Option<usize>,
) -> Result<()> {
    let index = load_index_or_report(engine)?;

    let search_limit = engine.resolve_search_limit(&std::env::current_dir()?, limit)?;
    let results = engine.search(&index.skills, query);
    let limited_results: Vec<_> = results.into_iter().take(search_limit).collect();

    let skills: Vec<_> = limited_results
        .iter()
        .map(|(skill, _)| SkillJsonView::from(*skill))
        .collect();
    print_json(&skills)?;

    Ok(())
}
