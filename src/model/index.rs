use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Serialized on-disk cache used by `sync`, `list`, `pick`, and `search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Index {
    pub(crate) version: String,
    pub(crate) last_sync: String,
    pub(crate) last_sync_unix: u64,
    pub(crate) config_fingerprint: String,
    pub(crate) skills: Vec<crate::model::Skill>,
}

impl Index {
    /// Create a new empty index stamped with the current package version and sync time.
    pub(crate) fn new(config_fingerprint: String) -> Self {
        let now = unix_timestamp();
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_sync: format!("{now}Z"),
            last_sync_unix: now,
            config_fingerprint,
            skills: Vec::new(),
        }
    }
}

fn unix_timestamp() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    duration.as_secs()
}

/// Parser failure captured during `sync --strict`.
#[derive(Debug, Clone)]
pub(crate) struct ParseError {
    pub(crate) path: String,
    pub(crate) reason: String,
}

impl ParseError {
    /// Create a parse error record for a single file.
    pub(crate) fn new(path: String, reason: String) -> Self {
        Self { path, reason }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.path, self.reason)
    }
}

impl std::error::Error for ParseError {}
