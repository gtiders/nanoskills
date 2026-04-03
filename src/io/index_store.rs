use crate::model::Index;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use super::config_loader::ConfigResolver;

const INDEX_FILE_NAME: &str = "index.json";

pub(crate) enum IndexLoadResult {
    Missing,
    Loaded(Index),
    Corrupted,
}

pub(crate) struct IndexStore {
    path: PathBuf,
}

impl IndexStore {
    pub(crate) fn new() -> Self {
        let cache_dir = ConfigResolver::get_cache_dir();
        Self {
            path: cache_dir.join(INDEX_FILE_NAME),
        }
    }

    pub(crate) fn load(&self) -> IndexLoadResult {
        if !self.path.exists() {
            return IndexLoadResult::Missing;
        }

        let content = match fs::read_to_string(&self.path) {
            Ok(content) => content,
            Err(_) => {
                return IndexLoadResult::Corrupted;
            }
        };

        match serde_json::from_str(&content) {
            Ok(index) => IndexLoadResult::Loaded(index),
            Err(_) => IndexLoadResult::Corrupted,
        }
    }

    pub(crate) fn save(&self, index: &Index) -> Result<()> {
        let cache_dir = ConfigResolver::ensure_cache_dir()?;
        let content = serde_json::to_string_pretty(index).context("failed to serialize index")?;
        let path = cache_dir.join(INDEX_FILE_NAME);

        fs::write(&path, content)
            .with_context(|| format!("failed to write index {}", path.display()))
    }
}
