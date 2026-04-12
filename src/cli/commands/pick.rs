use crate::services::SkillEngine;
use anyhow::Result;
use std::env::current_dir;

use super::shared::{load_index_or_report, run_picker};

/// Handle the default command with no explicit subcommand.
pub(crate) fn run_default_command(engine: &SkillEngine) -> Result<()> {
    let index = load_index_or_report(engine)?;
    let cwd = current_dir()?;
    let copy_to_clipboard = engine.copy_to_clipboard_on_pick(&cwd)?;

    run_picker(index.skills, copy_to_clipboard)
}

/// Handle `skillscripts pick`.
pub(crate) fn run_pick(engine: &SkillEngine) -> Result<()> {
    let index = load_index_or_report(engine)?;
    let cwd = current_dir()?;
    let copy_to_clipboard = engine.copy_to_clipboard_on_pick(&cwd)?;

    run_picker(index.skills, copy_to_clipboard)
}
