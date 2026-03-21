use crate::app::{LoadIndexState, SkillEngine};
use crate::domain::{Index, Skill};
use crate::presentation::output::print_skill_yaml_highlighted;
use crate::presentation::tui::run_tui;
use anyhow::Result;
use rust_i18n::t;

pub(crate) fn load_index_or_report(engine: &SkillEngine) -> Result<Option<Index>> {
    match engine.load_index() {
        LoadIndexState::Missing => {
            eprintln!("{}", t!("cli.index_not_found"));
            Ok(None)
        }
        LoadIndexState::Loaded(index) => Ok(Some(index)),
        LoadIndexState::Corrupted(error) => {
            eprintln!("{}", t!("cli.index_corrupted", reason = error));
            Ok(None)
        }
    }
}

pub(crate) fn run_picker(skills: Vec<Skill>) -> Result<()> {
    match run_tui(skills)? {
        Some(skill) => {
            print_skill_yaml_highlighted(&skill)?;
            println!("\n{}", t!("cli.selected_path", path = skill.path));
        }
        None => eprintln!("{}", t!("ui.no_selection")),
    }

    Ok(())
}
