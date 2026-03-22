use crate::domain::Config;
use crate::infra::{InitScope, get_global_config_dir, init_config, resolve_config};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(crate) struct ConfigService;

impl ConfigService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn init_global(&self, force: bool) -> Result<Config> {
        init_config(InitScope::Global, force)
    }

    pub(crate) fn init_local(&self, local_dir: &Path, force: bool) -> Result<Config> {
        init_config(InitScope::Local(local_dir.to_path_buf()), force)
    }

    pub(crate) fn global_config_dir(&self) -> PathBuf {
        get_global_config_dir()
    }

    pub(crate) fn resolve(&self, local_dir: &Path) -> Result<Config> {
        resolve_config(local_dir)
    }

    pub(crate) fn resolve_search_limit(
        &self,
        local_dir: &Path,
        requested_limit: Option<usize>,
    ) -> Result<usize> {
        Ok(requested_limit.unwrap_or(self.resolve(local_dir)?.search_limit))
    }

    pub(crate) fn detect_language(&self, local_dir: &Path) -> Option<String> {
        self.resolve(local_dir)
            .ok()
            .and_then(|config| config.language)
    }
}

pub(crate) fn detect_language(local_dir: &Path) -> Option<String> {
    ConfigService::new().detect_language(local_dir)
}
