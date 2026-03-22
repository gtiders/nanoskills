use crate::app::SkillEngine;
use anyhow::Result;
use rust_i18n::t;

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

    println!("{}", t!("cli.config_created", path = path.display()));
    println!(
        "{}",
        t!("cli.scan_paths", paths = format!("{:?}", config.scan_paths))
    );
    println!("{}", t!("cli.max_file_size", size = config.max_file_size));
    println!("{}", t!("cli.search_limit", limit = config.search_limit));

    Ok(())
}
