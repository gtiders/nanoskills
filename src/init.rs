use crate::portable_path::agent_skills_dir;
use crate::skill::{CREATE_SKILL, USE_SKILL};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const BUILTIN_SKILLS: &[(&str, &str)] = &[
    ("sks-script-use", USE_SKILL),
    ("sks-script-create", CREATE_SKILL),
];

pub(crate) struct SkillInstall {
    pub(crate) path: PathBuf,
    pub(crate) changed: bool,
}

pub(crate) fn install_agent_skills(force: bool) -> Result<Vec<SkillInstall>> {
    let skills_directory = agent_skills_dir()?;
    BUILTIN_SKILLS
        .iter()
        .map(|(name, content)| install_agent_skill(&skills_directory, name, content, force))
        .collect()
}

fn install_agent_skill(
    skills_directory: &std::path::Path,
    name: &str,
    content: &str,
    force: bool,
) -> Result<SkillInstall> {
    let directory = skills_directory.join(name);
    let path = directory.join("SKILL.md");
    if path.exists() && !force {
        return Ok(SkillInstall {
            path,
            changed: false,
        });
    }

    fs::create_dir_all(&directory)
        .with_context(|| format!("failed to create directory {}", directory.display()))?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write skill {}", path.display()))?;
    Ok(SkillInstall {
        path,
        changed: true,
    })
}
