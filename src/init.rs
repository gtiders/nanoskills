use crate::portable_path::agent_skills_dir;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const SKILL_NAME: &str = "sks-script-authoring";
const SKILL_CONTENT: &str = include_str!("../assets/skills/sks-script-authoring/SKILL.md");

pub(crate) struct SkillInstall {
    pub(crate) path: PathBuf,
    pub(crate) changed: bool,
}

pub(crate) fn install_agent_skill(force: bool) -> Result<SkillInstall> {
    let directory = agent_skills_dir()?.join(SKILL_NAME);
    let path = directory.join("SKILL.md");
    if path.exists() && !force {
        return Ok(SkillInstall {
            path,
            changed: false,
        });
    }

    fs::create_dir_all(&directory)
        .with_context(|| format!("failed to create directory {}", directory.display()))?;
    fs::write(&path, SKILL_CONTENT)
        .with_context(|| format!("failed to write skill {}", path.display()))?;
    Ok(SkillInstall {
        path,
        changed: true,
    })
}
