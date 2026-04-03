use crate::services::SkillEngine;
use anyhow::Result;

/// Handle `nanoskills config`.
pub(crate) fn run_config(engine: &SkillEngine) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let snapshot = engine.resolve_config_snapshot(&current_dir)?;

    println!("=== DEFAULT CONFIG ===");
    println!("{}", serde_yaml::to_string(&snapshot.default_config)?);

    println!("=== LOCAL CONFIG (CURRENT DIRECTORY) ===");
    match snapshot.local_config {
        Some(local) => println!("{}", serde_yaml::to_string(&local)?),
        None => println!("null"),
    }

    println!("=== EFFECTIVE CONFIG ===");
    println!("{}", serde_yaml::to_string(&snapshot.effective_config)?);

    Ok(())
}
