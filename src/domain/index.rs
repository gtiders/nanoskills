use serde::{Deserialize, Serialize};
use std::fmt;

/// Serialized on-disk cache used by `sync`, `list`, `pick`, and `search`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Index {
    pub(crate) version: String,
    pub(crate) last_sync: String,
    pub(crate) skills: Vec<crate::domain::Skill>,
}

impl Index {
    /// Create a new empty index stamped with the current package version and sync time.
    pub(crate) fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_sync: unix_timestamp_utc(),
            skills: Vec::new(),
        }
    }
}

fn unix_timestamp_utc() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{}Z", duration.as_secs())
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
