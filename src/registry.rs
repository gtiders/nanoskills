use crate::portable_path;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const CONFIG_FILE_NAME: &str = "sks.yaml";
pub(crate) const PATH_PLACEHOLDER: &str = "{{path}}";
pub(crate) const DEFAULT_MCP_SEARCH_LIMIT: usize = 5;
pub(crate) const MAX_MCP_SEARCH_LIMIT: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub(crate) struct ScriptId(pub(crate) u32);

impl fmt::Display for ScriptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for ScriptId {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u32>().map(Self)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) mcp: Option<McpConfig>,
    #[serde(default)]
    pub(crate) imports: Vec<String>,
    #[serde(default)]
    pub(crate) scripts: Vec<ScriptRegistration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct McpConfig {
    #[serde(default = "default_mcp_search_limit")]
    pub(crate) search_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ScriptRegistration {
    pub(crate) id: ScriptId,
    pub(crate) path: String,
    pub(crate) command: String,
    #[serde(default)]
    pub(crate) comment: Option<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Skill {
    pub(crate) id: ScriptId,
    #[serde(serialize_with = "serialize_path")]
    pub(crate) path: PathBuf,
    pub(crate) command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) comment: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) tags: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConfigSource {
    path: PathBuf,
    config: ConfigFile,
}

pub(crate) struct LoadedRegistry {
    pub(crate) skills: Vec<Skill>,
    pub(crate) mcp_search_limit: usize,
}

#[derive(Debug, Clone, Copy)]
struct PathResolver<'a> {
    base_dir: &'a Path,
}

pub(crate) fn global_config_dir() -> Result<PathBuf> {
    portable_path::config_dir()
}

pub(crate) fn global_config_path() -> Result<PathBuf> {
    Ok(global_config_dir()?.join(CONFIG_FILE_NAME))
}

pub(crate) fn init_global_config(force: bool) -> Result<bool> {
    let config_dir = global_config_dir()?;
    fs::create_dir_all(&config_dir)
        .with_context(|| format!("failed to create directory {}", config_dir.display()))?;

    let config_path = config_dir.join(CONFIG_FILE_NAME);
    let changed = !config_path.exists() || force;
    if changed {
        let content = serde_yaml::to_string(&default_global_config())
            .context("failed to serialize default config")?;
        fs::write(&config_path, content)
            .with_context(|| format!("failed to write config {}", config_path.display()))?;
    }

    let scripts_path = config_dir.join("scripts.yaml");
    if !scripts_path.exists() {
        fs::write(&scripts_path, "scripts: []\n")
            .with_context(|| format!("failed to write config {}", scripts_path.display()))?;
    }

    Ok(changed)
}

pub(crate) fn load_skills() -> Result<Vec<Skill>> {
    Ok(load_registry()?.skills)
}

pub(crate) fn load_registry() -> Result<LoadedRegistry> {
    let sources = load_config_sources()?;
    let mcp_search_limit = sources[0]
        .config
        .mcp
        .as_ref()
        .map_or(DEFAULT_MCP_SEARCH_LIMIT, |mcp| mcp.search_limit);
    let mut skills = build_skills(&sources)?;
    skills.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.path.cmp(&right.path))
    });
    validate_unique_ids(&skills)?;
    Ok(LoadedRegistry {
        skills,
        mcp_search_limit,
    })
}

pub(crate) fn display_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn load_config_sources() -> Result<Vec<ConfigSource>> {
    let global_path = global_config_path()?;
    let global = load_global_config_source(global_path)?;
    validate_mcp_config(&global.config)?;
    let global_base_dir = parent_dir(&global.path)?;
    let imports = global.config.imports.clone();
    let resolver = PathResolver {
        base_dir: global_base_dir.as_path(),
    };
    let mut sources = vec![global];

    for import in &imports {
        let import_path = resolver.resolve(import, "import")?;
        sources.push(load_imported_config_source(import_path)?);
    }

    Ok(sources)
}

fn load_global_config_source(path: PathBuf) -> Result<ConfigSource> {
    if !path.exists() {
        bail!(
            "Global config not found at {}. Run `sks init` first.",
            path.display()
        );
    }

    Ok(ConfigSource {
        config: load_config_file(&path)?,
        path,
    })
}

fn load_imported_config_source(path: PathBuf) -> Result<ConfigSource> {
    let config = load_config_file(&path)?;
    if !config.imports.is_empty() {
        bail!("Imported config {} cannot declare imports.", path.display());
    }
    if config.mcp.is_some() {
        bail!(
            "Imported config {} cannot declare mcp options.",
            path.display()
        );
    }

    Ok(ConfigSource { path, config })
}

fn build_skills(sources: &[ConfigSource]) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();
    for source in sources {
        skills.extend(build_skills_from_source(source)?);
    }
    Ok(skills)
}

fn build_skills_from_source(source: &ConfigSource) -> Result<Vec<Skill>> {
    let base_dir = parent_dir(&source.path)?;
    let resolver = PathResolver {
        base_dir: base_dir.as_path(),
    };

    source
        .config
        .scripts
        .iter()
        .map(|entry| build_skill(entry, &source.path, resolver))
        .collect()
}

fn build_skill(
    entry: &ScriptRegistration,
    config_path: &Path,
    resolver: PathResolver<'_>,
) -> Result<Skill> {
    let path = resolver
        .resolve(&entry.path, "script path")
        .with_context(|| format!("in {}", config_path.display()))?;

    if !path.is_file() {
        bail!(
            "Registered script {} points to a missing file: {}",
            entry.id,
            path.display()
        );
    }

    if entry.command.trim().is_empty() {
        bail!("Registered script {} has an empty command.", entry.id);
    }
    if !entry.command.contains(PATH_PLACEHOLDER) {
        bail!(
            "Registered script {} command must contain {}.",
            entry.id,
            PATH_PLACEHOLDER
        );
    }

    Ok(Skill {
        id: entry.id,
        path,
        command: entry.command.clone(),
        comment: entry.comment.clone(),
        tags: normalize_tags(entry.id, &entry.tags)?,
    })
}

fn load_config_file(path: &Path) -> Result<ConfigFile> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse YAML {}", path.display()))
}

impl PathResolver<'_> {
    fn resolve(self, value: &str, label: &str) -> Result<PathBuf> {
        let joined = portable_path::resolve_unix_relative(self.base_dir, value, label)?;
        if joined.exists() {
            return dunce::canonicalize(&joined)
                .with_context(|| format!("failed to canonicalize {}", joined.display()));
        }

        Ok(joined)
    }
}

fn normalize_tags(id: ScriptId, tags: &[String]) -> Result<Vec<String>> {
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if tag.is_empty() {
            bail!("Registered script {id} contains an empty tag.");
        }
        if !normalized
            .iter()
            .any(|existing: &String| existing.eq_ignore_ascii_case(tag))
        {
            normalized.push(tag.to_string());
        }
    }
    Ok(normalized)
}

fn validate_unique_ids(skills: &[Skill]) -> Result<()> {
    let mut seen: HashMap<ScriptId, &Path> = HashMap::new();
    for skill in skills {
        if let Some(existing) = seen.insert(skill.id, skill.path.as_path()) {
            bail!(
                "Duplicate id {} found in {} and {}",
                skill.id,
                display_path(existing),
                display_path(&skill.path)
            );
        }
    }
    Ok(())
}

fn validate_mcp_config(config: &ConfigFile) -> Result<()> {
    if let Some(mcp) = &config.mcp
        && !(1..=MAX_MCP_SEARCH_LIMIT).contains(&mcp.search_limit)
    {
        bail!(
            "mcp.search_limit must be between 1 and {MAX_MCP_SEARCH_LIMIT}, got {}.",
            mcp.search_limit
        );
    }
    Ok(())
}

fn parent_dir(path: &Path) -> Result<PathBuf> {
    path.parent()
        .map(Path::to_path_buf)
        .context("config file must have a parent directory")
}

fn default_global_config() -> ConfigFile {
    ConfigFile {
        mcp: Some(McpConfig {
            search_limit: DEFAULT_MCP_SEARCH_LIMIT,
        }),
        imports: vec!["scripts.yaml".to_string()],
        scripts: Vec::new(),
    }
}

const fn default_mcp_search_limit() -> usize {
    DEFAULT_MCP_SEARCH_LIMIT
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&display_path(path))
}
