use crate::io::{
    ConfigSnapshot, InitScope, get_global_config_dir, init_config, resolve_config,
    resolve_config_snapshot,
};
use crate::model::Config;
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

    pub(crate) fn resolve_snapshot(&self, local_dir: &Path) -> Result<ConfigSnapshot> {
        resolve_config_snapshot(local_dir)
    }

    pub(crate) fn resolve_search_limit(
        &self,
        local_dir: &Path,
        requested_limit: Option<usize>,
    ) -> Result<usize> {
        Ok(requested_limit.unwrap_or(self.resolve(local_dir)?.search_limit))
    }

}
