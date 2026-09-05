use anyhow::Result;

pub(crate) const USE_SKILL: &str = include_str!("../assets/skills/sks-script-use/SKILL.md");
pub(crate) const CREATE_SKILL: &str = include_str!("../assets/skills/sks-script-create/SKILL.md");

pub(crate) fn print_use() -> Result<()> {
    print!("{USE_SKILL}");
    Ok(())
}

pub(crate) fn print_create() -> Result<()> {
    print!("{CREATE_SKILL}");
    Ok(())
}
