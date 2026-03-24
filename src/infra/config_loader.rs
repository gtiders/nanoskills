use crate::domain::Config;
use anyhow::{Context, Result, bail};
use rust_i18n::t;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = ".agent-skills.yaml";
const SKILLS_DIR_NAME: &str = "skills";

pub(crate) enum InitScope {
    Global,
    Local(PathBuf),
}

/// Resolves nanoskills configuration from global and local scopes.
pub(crate) struct ConfigResolver {
    global_config_dir: PathBuf,
    local_config_dir: PathBuf,
}

impl ConfigResolver {
    /// Create a resolver rooted at the provided local working directory.
    pub(crate) fn new(local_dir: &Path) -> Self {
        let global_config_dir = global_config_dir();

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
        home_dir().join(".cache").join("nanoskills")
    }

    /// Ensure the cache directory exists before writing the index.
    pub(crate) fn ensure_cache_dir() -> Result<PathBuf> {
        let cache_dir = Self::get_cache_dir();
        fs::create_dir_all(&cache_dir)
            .with_context(|| format!("failed to create cache dir {}", cache_dir.display()))?;
        Ok(cache_dir)
    }
}

fn global_config_dir() -> PathBuf {
    home_dir().join(".config").join("nanoskills")
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn default_global_config() -> Config {
    Config {
        scan_paths: vec![SKILLS_DIR_NAME.to_string()],
        ..Config::default()
    }
}

fn default_local_config() -> Config {
    Config::default()
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
    let home = home_dir();
    for scan_path in &mut config.scan_paths {
        *scan_path = resolve_scan_path(base_dir, &home, scan_path);
    }
}

fn resolve_scan_path(base_dir: &Path, home: &Path, scan_path: &str) -> String {
    if scan_path == "~" {
        return home.to_string_lossy().into_owned();
    }
    if let Some(rest) = scan_path
        .strip_prefix("~/")
        .or_else(|| scan_path.strip_prefix("~\\"))
    {
        return home.join(rest).to_string_lossy().into_owned();
    }

    let candidate = Path::new(scan_path);
    if candidate.is_relative() {
        return base_dir.join(candidate).to_string_lossy().into_owned();
    }

    scan_path.to_string()
}

/// Return the global config directory under `~/.config/nanoskills`.
pub(crate) fn get_global_config_dir() -> PathBuf {
    global_config_dir()
}

pub(crate) fn init_config(scope: InitScope, force: bool) -> Result<Config> {
    let (config_dir, config) = match scope {
        InitScope::Global => {
            let config_dir = get_global_config_dir();
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("failed to create directory {}", config_dir.display()))?;
            let global_skills_dir = config_dir.join(SKILLS_DIR_NAME);
            let skills_dir_preexisted = global_skills_dir.exists();
            fs::create_dir_all(&global_skills_dir).with_context(|| {
                format!("failed to create skills directory {}", config_dir.display())
            })?;
            if !skills_dir_preexisted {
                seed_global_skills_from_workspace(&global_skills_dir)?;
            }
            (config_dir, default_global_config())
        }
        InitScope::Local(config_dir) => {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("failed to create directory {}", config_dir.display()))?;
            (config_dir, default_local_config())
        }
    };

    let config_path = config_dir.join(CONFIG_FILE_NAME);
    if config_path.exists() && !force {
        bail!("{}", t!("cli.config_exists", path = config_path.display()));
    }

    let content = serde_yaml::to_string(&config).context("failed to serialize default config")?;
    fs::write(&config_path, content)
        .with_context(|| format!("failed to write config {}", config_path.display()))?;

    Ok(config)
}

fn seed_global_skills_from_workspace(global_skills_dir: &Path) -> Result<()> {
    let Some(source_skills_dir) = find_seed_skills_source()? else {
        return Ok(());
    };

    copy_dir_contents_if_missing(&source_skills_dir, global_skills_dir)
}

fn copy_dir_contents_if_missing(source: &Path, destination: &Path) -> Result<()> {
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", source.display()))?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect {}", source_path.display()))?;

        if file_type.is_dir() {
            fs::create_dir_all(&destination_path)
                .with_context(|| format!("failed to create {}", destination_path.display()))?;
            copy_dir_contents_if_missing(&source_path, &destination_path)?;
            continue;
        }

        if file_type.is_file() && !destination_path.exists() {
            fs::copy(&source_path, &destination_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn find_seed_skills_source() -> Result<Option<PathBuf>> {
    // 发布后二进制场景：优先从可执行文件同级目录下的 skills 复制。
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let binary_adjacent = exe_dir.join(SKILLS_DIR_NAME);
            if binary_adjacent.is_dir() {
                return Ok(Some(binary_adjacent));
            }
        }
    }

    // 源码开发/本地调试场景：回退到当前工作目录下的 ./skills。
    let workspace_skills_dir = std::env::current_dir()
        .context("failed to resolve current working directory")?
        .join(SKILLS_DIR_NAME);
    if workspace_skills_dir.is_dir() {
        return Ok(Some(workspace_skills_dir));
    }

    Ok(None)
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

    #[test]
    fn test_resolve_scan_path_expands_tilde_and_keeps_absolute_path() {
        let base = Path::new("/workspace/project");
        let home = Path::new("/home/demo");

        assert_eq!(
            resolve_scan_path(base, home, "~/skills"),
            "/home/demo/skills"
        );
        assert_eq!(resolve_scan_path(base, home, "~"), "/home/demo");
        assert_eq!(
            resolve_scan_path(base, home, "relative/skills"),
            "/workspace/project/relative/skills"
        );
        assert_eq!(
            resolve_scan_path(base, home, "/opt/shared-skills"),
            "/opt/shared-skills"
        );
    }

    #[test]
    fn test_copy_dir_contents_if_missing_copies_nested_files_without_overwrite() {
        let temp = tempdir().expect("failed to create temp dir");
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source.join("nested")).expect("failed to create source nested dir");
        fs::create_dir_all(&destination).expect("failed to create destination dir");

        fs::write(source.join("a.md"), "from-source").expect("failed to write source file");
        fs::write(source.join("nested").join("b.md"), "from-source-nested")
            .expect("failed to write source nested file");
        fs::write(destination.join("a.md"), "existing").expect("failed to write destination file");

        copy_dir_contents_if_missing(&source, &destination).expect("copy should succeed");

        // 已存在文件不能被覆盖。
        assert_eq!(
            fs::read_to_string(destination.join("a.md")).expect("failed to read destination file"),
            "existing"
        );
        // 新文件和子目录文件应被复制。
        assert_eq!(
            fs::read_to_string(destination.join("nested").join("b.md"))
                .expect("failed to read copied nested file"),
            "from-source-nested"
        );
    }
}
