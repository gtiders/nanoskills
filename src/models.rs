use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub scan_paths: Vec<String>,
    #[serde(default = "default_file_patterns")]
    pub file_patterns: Vec<String>,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

fn default_file_patterns() -> Vec<String> {
    vec![
        "*.py".to_string(),
        "*.sh".to_string(),
        "*.js".to_string(),
        "*.ts".to_string(),
        "*.rs".to_string(),
        "*.go".to_string(),
        "*.lua".to_string(),
        "*.rb".to_string(),
        "*.c".to_string(),
        "*.cpp".to_string(),
        "*.scm".to_string(),
        "*.lisp".to_string(),
        "*.sql".to_string(),
        "*.f90".to_string(),
        "*.jl".to_string(),
        "*.pl".to_string(),
        "*.md".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scan_paths: vec![".".to_string()],
            file_patterns: default_file_patterns(),
            ignore_patterns: vec![],
        }
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
