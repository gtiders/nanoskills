use crate::domain::Index;
use crate::infra::{IndexLoadResult, IndexStore};
use anyhow::Result;

pub(crate) enum LoadIndexState {
    Missing,
    Loaded(Index),
    Corrupted(String),
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

    pub(crate) fn load(&self) -> LoadIndexState {
        match self.store.load() {
            IndexLoadResult::Missing => LoadIndexState::Missing,
            IndexLoadResult::Loaded(index) => LoadIndexState::Loaded(index),
            IndexLoadResult::Corrupted(error) => LoadIndexState::Corrupted(error.to_string()),
        }
    }

    pub(crate) fn save(&self, index: &Index) -> Result<()> {
        self.store.save(index)
    }
}
