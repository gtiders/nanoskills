use crate::models::{Config, Index, Skill};
use crate::parser::parse_header;
use crate::scanner::scan_files;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const INDEX_FILE_NAME: &str = ".skills-index.json";
const CONFIG_FILE_NAME: &str = ".agent-skills.yaml";

pub struct IndexManager {
    config: Config,
    index_path: PathBuf,
}

impl IndexManager {
    pub fn new(config: Config, index_dir: &Path) -> Self {
        let index_path = index_dir.join(INDEX_FILE_NAME);
        IndexManager { config, index_path }
    }

    pub fn build_index(&self) -> Result<Index> {
        let files = scan_files(&self.config)?;
        let mut skills = Vec::new();

        for file_path in files {
            let path = Path::new(&file_path);
            match parse_header(path) {
                Ok(Some(header)) => {
                    let skill = Skill::from((header, file_path));
                    skills.push(skill);
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("解析文件 {} 时出错: {}", file_path, e);
                }
            }
        }

        let mut index = Index::new();
        index.skills = skills;
        index.last_sync = current_timestamp();

        Ok(index)
    }

    pub fn load_index(&self) -> Result<Option<Index>> {
        if !self.index_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.index_path)?;
        let index: Index = serde_json::from_str(&content)?;
        Ok(Some(index))
    }

    pub fn save_index(&self, index: &Index) -> Result<()> {
        let content = serde_json::to_string_pretty(index)?;
        fs::write(&self.index_path, content)?;
        Ok(())
    }

    pub fn sync(&self) -> Result<Index> {
        let index = self.build_index()?;
        self.save_index(&index)?;
        Ok(index)
    }
}

pub struct SkillSearcher {
    index: Index,
}

impl SkillSearcher {
    pub fn new(index: Index) -> Self {
        SkillSearcher { index }
    }

    pub fn search(&self, query: &str) -> Vec<&Skill> {
        let query_lower = query.to_lowercase();

        self.index
            .skills
            .iter()
            .filter(|skill| {
                let name_match = skill.name.to_lowercase().contains(&query_lower);
                let desc_match = skill.description.to_lowercase().contains(&query_lower);
                let tag_match = skill
                    .tags
                    .iter()
                    .any(|t| t.to_lowercase().contains(&query_lower));

                name_match || desc_match || tag_match
            })
            .collect()
    }

    pub fn search_by_tags(&self, tags: &[String]) -> Vec<&Skill> {
        let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();

        self.index
            .skills
            .iter()
            .filter(|skill| {
                skill.tags.iter().any(|skill_tag| {
                    let skill_tag_lower = skill_tag.to_lowercase();
                    tags_lower.iter().any(|t| skill_tag_lower.contains(t))
                })
            })
            .collect()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Skill> {
        self.index.skills.iter().find(|s| s.name == name)
    }

    pub fn fuzzy_search(&self, query: &str) -> Vec<(&Skill, i64)> {
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        let mut results: Vec<(&Skill, i64)> = self
            .index
            .skills
            .iter()
            .filter_map(|skill| {
                let name_score = matcher.fuzzy_match(&skill.name, query).unwrap_or(0);
                let desc_score = matcher.fuzzy_match(&skill.description, query).unwrap_or(0);
                let max_score = name_score.max(desc_score / 2);

                if max_score > 0 {
                    Some((skill, max_score))
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| b.1.cmp(&a.1));
        results
    }
}

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let datetime = chrono_like_format(secs);
    format!("{}Z", datetime)
}

fn chrono_like_format(secs: u64) -> String {
    let days = secs / 86400;
    let years = 1970 + days / 365;
    let remaining_days = days % 365;
    let month = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        years, month, day, hours, minutes, seconds
    )
}

pub fn load_config(path: &Path) -> Result<Config> {
    let config_path = path.join(CONFIG_FILE_NAME);
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(path: &Path, config: &Config) -> Result<()> {
    let config_path = path.join(CONFIG_FILE_NAME);
    let content = serde_yaml::to_string(config)?;
    fs::write(&config_path, content)?;
    Ok(())
}

pub fn init_config(path: &Path) -> Result<Config> {
    let config = Config::default();
    save_config(path, &config)?;
    Ok(config)
}
