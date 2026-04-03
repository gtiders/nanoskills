use crate::services::SkillEngine;
use anyhow::Result;

/// Handle `nanoskills init`.
pub(crate) fn run_init(engine: &SkillEngine, force: bool, local: bool) -> Result<()> {
    let path = if local {
        std::env::current_dir()?
    } else {
        engine.global_config_dir()
    };
    let config = if local {
        engine.init_local_config(&path, force)?
    } else {
        engine.init_global_config(force)?
    };

    println!("Created {}/.agent-skills.yaml", path.display());
    println!("Scan Paths: {:?}", config.scan_paths);
    println!("Max File Size: {}", config.max_file_size);
    println!("Search Limit: {}", config.search_limit);

    Ok(())
}
