use crate::domain::Config;
use anyhow::{Context, Result, bail};
use rust_i18n::t;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = ".agent-skills.yaml";

/// Resolves nanoskills configuration from global and local scopes.
pub(crate) struct ConfigResolver {
    global_config_dir: PathBuf,
    local_config_dir: PathBuf,
}

impl ConfigResolver {
    /// Create a resolver rooted at the provided local working directory.
    pub(crate) fn new(local_dir: &Path) -> Self {
        let global_config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nanoskills");

        Self {
            global_config_dir,
            local_config_dir: local_dir.to_path_buf(),
        }
    }

    /// Merge global and local config files, then inject the default shared skills path.
    pub(crate) fn resolve(&self) -> Result<Config> {
        let global_config =
            self.load_optional_config(&self.global_config_dir.join(CONFIG_FILE_NAME))?;
        let local_config =
            self.load_optional_config(&self.local_config_dir.join(CONFIG_FILE_NAME))?;

        let mut config = match (global_config, local_config) {
            (Some(global), Some(local)) => global.merge(&local),
            (Some(global), None) => global,
            (None, Some(local)) => local,
            (None, None) => Config::default(),
        };

        self.add_default_skills_path(&mut config);

        Ok(config)
    }

    fn load_optional_config(&self, path: &Path) -> Result<Option<Config>> {
        if !path.exists() {
            return Ok(None);
        }

        load_config_file(path).map(Some)
    }

    fn add_default_skills_path(&self, config: &mut Config) {
        let default_skills_path = self.global_config_dir.join("skills");
        let default_skills_path = default_skills_path.to_string_lossy().into_owned();

        if !config.scan_paths.contains(&default_skills_path) {
            config.scan_paths.insert(0, default_skills_path);
        }
    }

    /// Return the cache directory used for the generated index.
    pub(crate) fn get_cache_dir() -> PathBuf {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nanoskills")
    }

    /// Ensure the cache directory exists before writing the index.
    pub(crate) fn ensure_cache_dir() -> Result<PathBuf> {
        let cache_dir = Self::get_cache_dir();
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("failed to create cache dir {}", cache_dir.display()))?;
        Ok(cache_dir)
    }
}

fn load_config_file(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let mut config: Config = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse YAML {}", path.display()))?;
    absolutize_scan_paths(path.parent().unwrap_or_else(|| Path::new(".")), &mut config);
    Ok(config)
}

fn absolutize_scan_paths(base_dir: &Path, config: &mut Config) {
    for scan_path in &mut config.scan_paths {
        let candidate = Path::new(scan_path);
        if candidate.is_relative() {
            *scan_path = base_dir.join(candidate).to_string_lossy().into_owned();
        }
    }
}

/// Initialize a new `.agent-skills.yaml` under the provided directory.
pub(crate) fn init_config(path: &Path, force: bool) -> Result<Config> {
    fs::create_dir_all(path)
        .with_context(|| format!("failed to create directory {}", path.display()))?;

    let config_path = path.join(CONFIG_FILE_NAME);
    if config_path.exists() && !force {
        bail!("{}", t!("cli.config_exists", path = config_path.display()));
    }

    let config = Config::default();
    let content = serde_yaml::to_string(&config).context("failed to serialize default config")?;
    fs::write(&config_path, content)
        .with_context(|| format!("failed to write config {}", config_path.display()))?;

    Ok(config)
}

/// Resolve nanoskills config relative to a local working directory.
pub(crate) fn resolve_config(local_dir: &Path) -> Result<Config> {
    ConfigResolver::new(local_dir).resolve()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_merges_global_local_and_absolutizes_relative_paths() {
        let temp = tempdir().expect("failed to create temp dir");
        let global_dir = temp.path().join("global");
        let local_dir = temp.path().join("workspace");
        fs::create_dir_all(&global_dir).expect("failed to create global dir");
        fs::create_dir_all(&local_dir).expect("failed to create local dir");

        fs::write(
            global_dir.join(CONFIG_FILE_NAME),
            r#"
scan_paths:
  - shared
ignore_patterns:
  - target
search_limit: 8
"#,
        )
        .expect("failed to write global config");

        fs::write(
            local_dir.join(CONFIG_FILE_NAME),
            r#"
scan_paths:
  - ./skills
ignore_patterns:
  - dist
max_file_size: 2MB
language: zh-CN
"#,
        )
        .expect("failed to write local config");

        let resolver = ConfigResolver {
            global_config_dir: global_dir.clone(),
            local_config_dir: local_dir.clone(),
        };
        let config = resolver.resolve().expect("config should resolve");

        let default_skills_path = global_dir.join("skills").to_string_lossy().into_owned();
        let global_scan_path = global_dir.join("shared").to_string_lossy().into_owned();
        let local_scan_path = local_dir.join("./skills").to_string_lossy().into_owned();

        // 解析后路径必须全部绝对化，并且保留默认全局 skills 注入逻辑。
        assert_eq!(
            config.scan_paths,
            vec![default_skills_path, global_scan_path, local_scan_path]
        );
        // 全局和本地列表字段应合并，本地标量覆盖全局。
        assert_eq!(config.ignore_patterns, vec!["target", "dist"]);
        assert_eq!(config.search_limit, 8);
        assert_eq!(config.max_file_size, 2 * 1024 * 1024);
        assert_eq!(config.language.as_deref(), Some("zh-CN"));
    }

    #[test]
    fn test_load_config_file_reports_yaml_errors_with_context() {
        let temp = tempdir().expect("failed to create temp dir");
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(&config_path, "scan_paths: [unterminated").expect("failed to write bad config");

        let error = load_config_file(&config_path).expect_err("invalid YAML should fail");

        // 用户看到的错误需要带上具体文件路径，否则坏配置很难定位。
        assert!(error.to_string().contains("failed to parse YAML"));
        assert!(error.to_string().contains(CONFIG_FILE_NAME));
    }
}
