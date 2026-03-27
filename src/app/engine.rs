use crate::app::{SyncResult, build_index, search::fuzzy_search};
use crate::app::{config_service::ConfigService, index_service::IndexService};
use crate::domain::{Config, ParseError, Skill};
use crate::infra::{ConfigSnapshot, scan_files};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub(crate) use crate::app::index_service::LoadIndexState;

pub(crate) struct SkillEngine {
    config_service: ConfigService,
    index_service: IndexService,
}

impl SkillEngine {
    pub(crate) fn new() -> Self {
        Self {
            config_service: ConfigService::new(),
            index_service: IndexService::new(),
        }
    }

    pub(crate) fn sync(&self, local_dir: &Path, strict: bool) -> Result<SyncResult> {
        let started_at = Instant::now();
        let config = self.config_service.resolve(local_dir)?;
        let files = scan_files(&config)?;
        let total_files = files.len();

        let (index, errors): (_, Vec<ParseError>) = build_index(&files, strict);
        let skills_count = index.skills.len();

        self.index_service.save(&index)?;

        Ok(SyncResult {
            total_files,
            skills_count,
            elapsed_ms: started_at.elapsed().as_millis(),
            errors,
        })
    }

    pub(crate) fn init_global_config(&self, force: bool) -> Result<Config> {
        self.config_service.init_global(force)
    }

    pub(crate) fn init_local_config(&self, local_dir: &Path, force: bool) -> Result<Config> {
        self.config_service.init_local(local_dir, force)
    }

    pub(crate) fn global_config_dir(&self) -> PathBuf {
        self.config_service.global_config_dir()
    }

    pub(crate) fn load_index(&self) -> LoadIndexState {
        self.index_service.load()
    }

    pub(crate) fn resolve_config_snapshot(&self, local_dir: &Path) -> Result<ConfigSnapshot> {
        self.config_service.resolve_snapshot(local_dir)
    }

    pub(crate) fn resolve_search_limit(
        &self,
        local_dir: &Path,
        requested_limit: Option<usize>,
    ) -> Result<usize> {
        self.config_service
            .resolve_search_limit(local_dir, requested_limit)
    }

    pub(crate) fn search<'a>(&self, skills: &'a [Skill], query: &str) -> Vec<(&'a Skill, i64)> {
        fuzzy_search(skills, query)
    }
}
