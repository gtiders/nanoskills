use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub(crate) struct TestEnv {
    temp_dir: TempDir,
    cache_dir: PathBuf,
    config_dir: PathBuf,
    home_dir: PathBuf,
}

impl TestEnv {
    pub(crate) fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let cache_dir = temp_dir.path().join("xdg-cache");
        let config_dir = temp_dir.path().join("xdg-config");
        let home_dir = temp_dir.path().join("home");

        fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        fs::create_dir_all(&config_dir).expect("failed to create config dir");
        fs::create_dir_all(&home_dir).expect("failed to create home dir");

        Self {
            temp_dir,
            cache_dir,
            config_dir,
            home_dir,
        }
    }

    pub(crate) fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    pub(crate) fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub(crate) fn command_envs(&self) -> Vec<(&'static str, &Path)> {
        vec![
            ("XDG_CACHE_HOME", self.cache_dir()),
            ("XDG_CONFIG_HOME", &self.config_dir),
            ("HOME", &self.home_dir),
        ]
    }

    pub(crate) fn command(&self, workspace: &Path) -> Command {
        let mut cmd = Command::cargo_bin("nanoskills").expect("binary should build");
        cmd.current_dir(workspace);
        for (key, value) in self.command_envs() {
            cmd.env(key, value);
        }
        cmd.env("LANG", "en_US.UTF-8");
        cmd.env("LC_ALL", "en_US.UTF-8");
        cmd
    }

    #[allow(dead_code)]
    pub(crate) fn run_sync(&self, workspace: &Path) {
        self.command(workspace).arg("sync").assert().success();
    }
}
