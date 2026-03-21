use crate::app::SkillEngine;
use anyhow::Result;
use rust_i18n::t;

/// Handle `nanoskills init`.
pub(crate) fn run_init(engine: &SkillEngine, force: bool) -> Result<()> {
    let path = std::env::current_dir()?;
    let config = engine.init_config(&path, force)?;

    println!("{}", t!("cli.config_created", path = path.display()));
    println!(
        "{}",
        t!("cli.scan_paths", paths = format!("{:?}", config.scan_paths))
    );
    println!("{}", t!("cli.max_file_size", size = config.max_file_size));
    println!("{}", t!("cli.search_limit", limit = config.search_limit));

    Ok(())
}
