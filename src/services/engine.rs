use crate::io::{ConfigSnapshot, scan_files};
use crate::model::{Config, Index, ParseError, Skill};
use crate::services::index_service::IndexRefreshState;
use crate::services::{SyncResult, build_index, search::fuzzy_search};
use crate::services::{config_service::ConfigService, index_service::IndexService};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub(crate) struct SkillEngine {
    config_service: ConfigService,
    index_service: IndexService,
}

pub(crate) enum CacheRefreshReason {
    Missing,
    Corrupted,
    Stale { ttl_seconds: u64 },
    ConfigChanged,
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
        let config_fingerprint = config.fingerprint();
        let files = scan_files(&config)?;
        let total_files = files.len();

        let (index, errors): (_, Vec<ParseError>) =
            build_index(&files, strict, config_fingerprint);
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

    pub(crate) fn ensure_index(
        &self,
        local_dir: &Path,
    ) -> Result<(Index, Option<CacheRefreshReason>)> {
        let config = self.config_service.resolve(local_dir)?;
        let expected_fingerprint = config.fingerprint();
        let now = current_unix_timestamp();

        let check = self
            .index_service
            .evaluate_refresh_state(now, config.cache_ttl_seconds, &expected_fingerprint);

        let reason = match check {
            IndexRefreshState::Usable(index) => return Ok((index, None)),
            IndexRefreshState::Missing => CacheRefreshReason::Missing,
            IndexRefreshState::Corrupted => CacheRefreshReason::Corrupted,
            IndexRefreshState::Stale => CacheRefreshReason::Stale {
                ttl_seconds: config.cache_ttl_seconds,
            },
            IndexRefreshState::ConfigChanged => CacheRefreshReason::ConfigChanged,
        };

        let files = scan_files(&config)?;
        let (index, _errors): (_, Vec<ParseError>) =
            build_index(&files, false, expected_fingerprint);
        self.index_service.save(&index)?;
        Ok((index, Some(reason)))
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

    pub(crate) fn copy_to_clipboard_on_pick(&self, local_dir: &Path) -> Result<bool> {
        let config = self.config_service.resolve(local_dir)?;
        Ok(config.copy_to_clipboard_on_pick)
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
