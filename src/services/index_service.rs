use crate::io::{IndexLoadResult, IndexStore};
use crate::model::Index;
use anyhow::Result;

pub(crate) enum IndexRefreshState {
    Usable(Index),
    Missing,
    Corrupted,
    Stale,
    ConfigChanged,
}

pub(crate) struct IndexService {
    store: IndexStore,
}

impl IndexService {
    pub(crate) fn new() -> Self {
        Self {
            store: IndexStore::new(),
        }
    }

    pub(crate) fn save(&self, index: &Index) -> Result<()> {
        self.store.save(index)
    }

    pub(crate) fn evaluate_refresh_state(
        &self,
        now_unix: u64,
        ttl_seconds: u64,
        expected_fingerprint: &str,
    ) -> IndexRefreshState {
        match self.store.load() {
            IndexLoadResult::Missing => IndexRefreshState::Missing,
            IndexLoadResult::Corrupted => IndexRefreshState::Corrupted,
            IndexLoadResult::Loaded(index) => {
                if index.config_fingerprint != expected_fingerprint {
                    IndexRefreshState::ConfigChanged
                } else if is_stale(now_unix, index.last_sync_unix, ttl_seconds) {
                    IndexRefreshState::Stale
                } else {
                    IndexRefreshState::Usable(index)
                }
            }
        }
    }
}

fn is_stale(now_unix: u64, last_sync_unix: u64, ttl_seconds: u64) -> bool {
    if ttl_seconds == 0 {
        return false;
    }

    now_unix.saturating_sub(last_sync_unix) > ttl_seconds
}
