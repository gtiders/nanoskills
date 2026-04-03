use crate::cli::output::print_skill_yaml;
use crate::cli::picker::run_skim_picker;
use crate::model::{Index, Skill};
use crate::services::{CacheRefreshReason, SkillEngine};
use anyhow::Result;

pub(crate) fn load_index_or_report(engine: &SkillEngine) -> Result<Index> {
    let cwd = std::env::current_dir()?;
    let (index, refresh_reason) = engine.ensure_index(&cwd)?;

    if let Some(reason) = refresh_reason {
        match reason {
            CacheRefreshReason::Missing => {
                eprintln!("Local index cache is missing. Rebuilding now...");
            }
            CacheRefreshReason::Corrupted => {
                eprintln!("Local index cache is corrupted. Rebuilding now...");
            }
            CacheRefreshReason::Stale { ttl_seconds } => {
                eprintln!("Local index cache is stale (ttl={ttl_seconds}s). Rebuilding now...");
            }
            CacheRefreshReason::ConfigChanged => {
                eprintln!("Local config changed since last index build. Rebuilding now...");
            }
        }
    }

    Ok(index)
}

pub(crate) fn run_picker(skills: Vec<Skill>) -> Result<()> {
    match run_skim_picker(skills)? {
        Some(skill) => {
            print_skill_yaml(&skill)?;
            println!("\nSkill Path: {}", skill.path);
        }
        None => eprintln!("No skill selected."),
    }

    Ok(())
}
