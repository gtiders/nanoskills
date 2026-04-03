use crate::cli::output::print_sync_result;
use crate::services::SkillEngine;
use anyhow::Result;

/// Handle `nanoskills sync`.
pub(crate) fn run_sync(engine: &SkillEngine, strict: bool) -> Result<()> {
    let path = std::env::current_dir()?;
    let result = engine.sync(&path, strict)?;
    print_sync_result(&result);
    Ok(())
}
