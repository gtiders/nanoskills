use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub(crate) struct TestEnv {
    temp_dir: TempDir,
    config_dir: PathBuf,
    home_dir: PathBuf,
}

impl TestEnv {
    pub(crate) fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let home_dir = temp_dir.path().join("home");
        let cache_dir = home_dir.join(".cache");
        let config_dir = home_dir.join(".config");

        fs::create_dir_all(&cache_dir).expect("failed to create cache dir");
        fs::create_dir_all(&config_dir).expect("failed to create config dir");
        fs::create_dir_all(&home_dir).expect("failed to create home dir");

        Self {
            temp_dir,
            config_dir,
            home_dir,
        }
    }

    pub(crate) fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    pub(crate) fn cache_dir(&self) -> PathBuf {
        self.home_dir.join(".cache")
    }

    pub(crate) fn global_config_dir(&self) -> PathBuf {
        self.config_dir.join("skillscripts")
    }

    pub(crate) fn global_config_file(&self) -> PathBuf {
        self.global_config_dir().join("skillscripts.yaml")
    }

    pub(crate) fn command_envs(&self) -> Vec<(&'static str, &Path)> {
        vec![
            ("HOME", &self.home_dir),
            ("USERPROFILE", &self.home_dir),
        ]
    }

    pub(crate) fn command(&self, workspace: &Path) -> Command {
        let mut cmd = Command::cargo_bin("skillscripts").expect("binary should build");
        let _ = self.cache_dir();
        let _ = self.global_config_dir();
        let _ = self.global_config_file();
        cmd.current_dir(workspace);
        for (key, value) in self.command_envs() {
            cmd.env(key, value);
        }
        cmd.env("LANG", "en_US.UTF-8");
        cmd.env("LC_ALL", "en_US.UTF-8");
        cmd
    }

}
