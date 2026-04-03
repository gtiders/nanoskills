use crate::services::SkillEngine;
use anyhow::Result;

use super::shared::{load_index_or_report, run_picker};

/// Handle the default command with no explicit subcommand.
pub(crate) fn run_default_command(engine: &SkillEngine) -> Result<()> {
    let index = load_index_or_report(engine)?;

    run_picker(index.skills)
}

/// Handle `nanoskills pick`.
pub(crate) fn run_pick(engine: &SkillEngine) -> Result<()> {
    let index = load_index_or_report(engine)?;

    run_picker(index.skills)
}
