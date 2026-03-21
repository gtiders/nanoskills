use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024;
const DEFAULT_SEARCH_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHeader {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub command_template: Option<String>,
    #[serde(default)]
    pub args: HashMap<String, ArgDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgDef {
    #[serde(rename = "type")]
    pub arg_type: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub command_template: Option<String>,
    pub parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

impl From<(SkillHeader, String)> for Skill {
    fn from((header, path): (SkillHeader, String)) -> Self {
        let parameters = if !header.args.is_empty() {
            Some(build_parameters_schema(&header.args))
        } else {
            None
        };

        Skill {
            name: header.name,
            description: header.description,
            path,
            tags: header.tags,
            command_template: header.command_template,
            parameters,
            checksum: None,
        }
    }
}

fn build_parameters_schema(args: &HashMap<String, ArgDef>) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();

    for (name, def) in args {
        properties.insert(
            name.clone(),
            serde_json::json!({
                "type": def.arg_type,
                "description": def.description
            }),
        );
        if def.required {
            required.push(name.clone());
        }
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

fn parse_file_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();

    if let Ok(bytes) = s.parse::<u64>() {
        return Some(bytes);
    }

    let (num_str, multiplier) = if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024u64 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024u64 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024u64)
    } else if s.ends_with("G") {
        (&s[..s.len() - 1], 1024u64 * 1024 * 1024)
    } else if s.ends_with("M") {
        (&s[..s.len() - 1], 1024u64 * 1024)
    } else if s.ends_with("K") {
        (&s[..s.len() - 1], 1024u64)
    } else {
        return s.parse::<u64>().ok();
    };

    num_str.trim().parse::<u64>().ok().map(|n| n * multiplier)
}

fn serialize_file_size<S>(size: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mb = *size / (1024 * 1024);
    if (*size).is_multiple_of(1024 * 1024) {
        serializer.serialize_str(&format!("{}MB", mb))
    } else {
        serializer.serialize_u64(*size)
    }
}

fn deserialize_file_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let s = String::deserialize(deserializer)?;
    parse_file_size(&s).ok_or_else(|| D::Error::custom(format!("Invalid file size format: {}", s)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub scan_paths: Vec<String>,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
    #[serde(
        default = "default_max_file_size",
        serialize_with = "serialize_file_size",
        deserialize_with = "deserialize_file_size"
    )]
    pub max_file_size: u64,
    #[serde(default = "default_search_limit")]
    pub search_limit: usize,
    #[serde(default)]
    pub language: Option<String>,
}

fn default_max_file_size() -> u64 {
    DEFAULT_MAX_FILE_SIZE
}

fn default_search_limit() -> usize {
    DEFAULT_SEARCH_LIMIT
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scan_paths: vec![],
            ignore_patterns: vec![],
            max_file_size: DEFAULT_MAX_FILE_SIZE,
            search_limit: DEFAULT_SEARCH_LIMIT,
            language: None,
        }
    }
}

impl Config {
    pub fn merge(&self, other: &Config) -> Config {
        let mut merged = self.clone();

        if !other.scan_paths.is_empty() {
            for path in &other.scan_paths {
                if !merged.scan_paths.contains(path) {
                    merged.scan_paths.push(path.clone());
                }
            }
        }

        for pattern in &other.ignore_patterns {
            if !merged.ignore_patterns.contains(pattern) {
                merged.ignore_patterns.push(pattern.clone());
            }
        }

        if other.max_file_size != DEFAULT_MAX_FILE_SIZE {
            merged.max_file_size = other.max_file_size;
        }

        if other.search_limit != DEFAULT_SEARCH_LIMIT {
            merged.search_limit = other.search_limit;
        }

        if other.language.is_some() {
            merged.language = other.language.clone();
        }

        merged
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub version: String,
    pub last_sync: String,
    pub skills: Vec<Skill>,
}

impl Index {
    pub fn new() -> Self {
        Index {
            version: "1.0.0".to_string(),
            last_sync: chrono_lite_now(),
            skills: Vec::new(),
        }
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}Z", secs)
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub path: String,
    pub reason: String,
}

impl ParseError {
    pub fn new(path: String, reason: String) -> Self {
        ParseError { path, reason }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunction,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

impl From<&Skill> for OpenAITool {
    fn from(skill: &Skill) -> Self {
        OpenAITool {
            tool_type: "function".to_string(),
            function: OpenAIFunction {
                name: skill.name.clone(),
                description: skill.description.clone(),
                parameters: skill.parameters.clone(),
            },
        }
    }
}
