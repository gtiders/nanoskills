use crate::models::Config;
use anyhow::{Result, bail};
use rust_i18n::t;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = ".agent-skills.yaml";

pub struct ConfigResolver {
    global_config_dir: PathBuf,
    local_config_dir: PathBuf,
}

impl ConfigResolver {
    pub fn new(local_dir: &Path) -> Self {
        let global_config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nanoskills");

        ConfigResolver {
            global_config_dir,
            local_config_dir: local_dir.to_path_buf(),
        }
    }

    pub fn resolve(&self) -> Result<Config> {
        let global_config = self.load_global_config();
        let local_config = self.load_local_config();

        let mut config = match (global_config, local_config) {
            (Some(global), Some(local)) => global.merge(&local),
            (Some(global), None) => global,
            (None, Some(local)) => local,
            (None, None) => Config::default(),
        };

        self.add_default_skills_path(&mut config);

        Ok(config)
    }

    fn load_global_config(&self) -> Option<Config> {
        let config_path = self.global_config_dir.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            load_config_file(&config_path).ok()
        } else {
            None
        }
    }

    fn load_local_config(&self) -> Option<Config> {
        let config_path = self.local_config_dir.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            load_config_file(&config_path).ok()
        } else {
            None
        }
    }

    fn add_default_skills_path(&self, config: &mut Config) {
        let default_skills_path = self.global_config_dir.join("skills");
        let default_skills_str = default_skills_path.to_string_lossy().to_string();

        if !config.scan_paths.contains(&default_skills_str) {
            config.scan_paths.insert(0, default_skills_str);
        }
    }

    pub fn get_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nanoskills")
    }

    pub fn ensure_cache_dir() -> Result<PathBuf> {
        let cache_dir = Self::get_cache_dir();
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }
        Ok(cache_dir)
    }
}

fn load_config_file(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_yaml::from_str(&content)?;
    Ok(config)
}

pub fn init_config(path: &Path, force: bool) -> Result<Config> {
    let config_path = path.join(CONFIG_FILE_NAME);

    if config_path.exists() && !force {
        bail!("{}", t!("cli.config_exists", path = config_path.display()));
    }

    let config = Config::default();
    let content = serde_yaml::to_string(&config)?;
    fs::write(&config_path, content)?;
    Ok(config)
}

pub fn resolve_config(local_dir: &Path) -> Result<Config> {
    let resolver = ConfigResolver::new(local_dir);
    resolver.resolve()
}
