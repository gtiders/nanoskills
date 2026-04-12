use serde::{Deserialize, Serialize};

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;
const DEFAULT_SEARCH_LIMIT: usize = 5;
const DEFAULT_CACHE_TTL_SECONDS: u64 = 3600;

/// User configuration loaded from `skillscripts.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) scan_paths: Vec<String>,
    #[serde(default)]
    pub(crate) ignore_patterns: Vec<String>,
    #[serde(
        default = "default_max_file_size",
        serialize_with = "serialize_file_size",
        deserialize_with = "deserialize_file_size"
    )]
    pub(crate) max_file_size: u64,
    #[serde(default = "default_search_limit")]
    pub(crate) search_limit: usize,
    #[serde(
        default = "default_cache_ttl_seconds",
        serialize_with = "serialize_duration_seconds",
        deserialize_with = "deserialize_duration_seconds"
    )]
    pub(crate) cache_ttl_seconds: u64,
    #[serde(default)]
    pub(crate) copy_to_clipboard_on_pick: bool,
}

fn default_max_file_size() -> u64 {
    DEFAULT_MAX_FILE_SIZE
}

fn default_search_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

fn default_cache_ttl_seconds() -> u64 {
    DEFAULT_CACHE_TTL_SECONDS
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_paths: vec![".".to_string()],
            ignore_patterns: Vec::new(),
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            search_limit: DEFAULT_SEARCH_LIMIT,
            cache_ttl_seconds: DEFAULT_CACHE_TTL_SECONDS,
            copy_to_clipboard_on_pick: false,
        }
    }
}

impl Config {
    /// Merge another config into the current one, preserving unique list items
    /// while allowing scalar overrides from the more specific config.
    pub(crate) fn merge(&self, other: &Config) -> Config {
        let mut merged = self.clone();

        extend_unique(&mut merged.scan_paths, &other.scan_paths);
        extend_unique(&mut merged.ignore_patterns, &other.ignore_patterns);

        if other.max_file_size != DEFAULT_MAX_FILE_SIZE {
            merged.max_file_size = other.max_file_size;
        }

        if other.search_limit != DEFAULT_SEARCH_LIMIT {
            merged.search_limit = other.search_limit;
        }

        if other.cache_ttl_seconds != DEFAULT_CACHE_TTL_SECONDS {
            merged.cache_ttl_seconds = other.cache_ttl_seconds;
        }

        if other.copy_to_clipboard_on_pick {
            merged.copy_to_clipboard_on_pick = other.copy_to_clipboard_on_pick;
        }

        merged
    }

    pub(crate) fn fingerprint(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

fn extend_unique(target: &mut Vec<String>, source: &[String]) {
    for item in source {
        if !target.contains(item) {
            target.push(item.clone());
        }
    }
}

fn parse_file_size(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(bytes) = trimmed.parse::<u64>() {
        return Some(bytes);
    }

    let upper = trimmed.to_ascii_uppercase();
    let (number, multiplier) = if let Some(number) = upper.strip_suffix("GB") {
        (number, 1024u64 * 1024 * 1024)
    } else if let Some(number) = upper.strip_suffix("MB") {
        (number, 1024u64 * 1024)
    } else if let Some(number) = upper.strip_suffix("KB") {
        (number, 1024u64)
    } else if let Some(number) = upper.strip_suffix('G') {
        (number, 1024u64 * 1024 * 1024)
    } else if let Some(number) = upper.strip_suffix('M') {
        (number, 1024u64 * 1024)
    } else if let Some(number) = upper.strip_suffix('K') {
        (number, 1024u64)
    } else {
        return None;
    };

    number
        .trim()
        .parse::<u64>()
        .ok()
        .map(|size| size * multiplier)
}

fn parse_duration_seconds(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(seconds) = trimmed.parse::<u64>() {
        return Some(seconds);
    }

    let upper = trimmed.to_ascii_uppercase();
    let (number, multiplier) = if let Some(number) = upper.strip_suffix("MS") {
        let millis = number.trim().parse::<u64>().ok()?;
        return Some(millis / 1000);
    } else if let Some(number) = upper.strip_suffix('S') {
        (number, 1u64)
    } else if let Some(number) = upper.strip_suffix('M') {
        (number, 60u64)
    } else if let Some(number) = upper.strip_suffix('H') {
        (number, 60u64 * 60)
    } else if let Some(number) = upper.strip_suffix('D') {
        (number, 60u64 * 60 * 24)
    } else {
        return None;
    };

    number.trim().parse::<u64>().ok().map(|size| size * multiplier)
}

fn serialize_file_size<S>(size: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if size.is_multiple_of(1024 * 1024) {
        serializer.serialize_str(&format!("{}MB", size / (1024 * 1024)))
    } else {
        serializer.serialize_u64(*size)
    }
}

fn deserialize_file_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum FileSizeValue {
        Number(u64),
        String(String),
    }

    match FileSizeValue::deserialize(deserializer)? {
        FileSizeValue::Number(size) => Ok(size),
        FileSizeValue::String(size) => parse_file_size(&size)
            .ok_or_else(|| D::Error::custom(format!("Invalid file size format: {size}"))),
    }
}

fn serialize_duration_seconds<S>(seconds: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if *seconds != 0 && seconds.is_multiple_of(60 * 60 * 24) {
        serializer.serialize_str(&format!("{}d", seconds / (60 * 60 * 24)))
    } else if *seconds != 0 && seconds.is_multiple_of(60 * 60) {
        serializer.serialize_str(&format!("{}h", seconds / (60 * 60)))
    } else if *seconds != 0 && seconds.is_multiple_of(60) {
        serializer.serialize_str(&format!("{}m", seconds / 60))
    } else {
        serializer.serialize_u64(*seconds)
    }
}

fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DurationValue {
        Number(u64),
        String(String),
    }

    match DurationValue::deserialize(deserializer)? {
        DurationValue::Number(seconds) => Ok(seconds),
        DurationValue::String(raw) => parse_duration_seconds(&raw)
            .ok_or_else(|| D::Error::custom(format!("Invalid duration format: {raw}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_combines_unique_paths_and_overrides_local_scalars() {
        let global = Config {
            scan_paths: vec!["/global/skills".into(), "/shared".into()],
            ignore_patterns: vec!["target".into()],
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            search_limit: DEFAULT_SEARCH_LIMIT,
            cache_ttl_seconds: DEFAULT_CACHE_TTL_SECONDS,
            copy_to_clipboard_on_pick: true,
        };
        let local = Config {
            scan_paths: vec!["/shared".into(), "./local".into()],
            ignore_patterns: vec!["dist".into()],
            max_file_size: 2 * 1024 * 1024,
            search_limit: 9,
            cache_ttl_seconds: 120,
            copy_to_clipboard_on_pick: true,
        };

        let merged = global.merge(&local);

        assert_eq!(
            merged.scan_paths,
            vec!["/global/skills", "/shared", "./local"]
        );
        assert_eq!(merged.ignore_patterns, vec!["target", "dist"]);
        assert_eq!(merged.max_file_size, 2 * 1024 * 1024);
        assert_eq!(merged.search_limit, 9);
        assert_eq!(merged.cache_ttl_seconds, 120);
        assert!(merged.copy_to_clipboard_on_pick);
    }

    #[test]
    fn test_merge_keeps_global_scalars_when_local_uses_defaults() {
        let global = Config {
            scan_paths: vec!["/global/skills".into()],
            ignore_patterns: vec![],
            max_file_size: 2 * 1024 * 1024,
            search_limit: 20,
            cache_ttl_seconds: 7200,
            copy_to_clipboard_on_pick: true,
        };
        let local = Config::default();

        let merged = global.merge(&local);

        assert_eq!(merged.max_file_size, 2 * 1024 * 1024);
        assert_eq!(merged.search_limit, 20);
        assert_eq!(merged.cache_ttl_seconds, 7200);
        assert!(merged.copy_to_clipboard_on_pick);
    }
}
