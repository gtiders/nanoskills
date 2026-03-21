use crate::domain::build_auto_tool_name;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Parsed skill frontmatter/header shared by all supported source languages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SkillHeader {
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) version: Option<String>,
    #[serde(default)]
    pub(crate) command_template: Option<String>,
    #[serde(default)]
    pub(crate) tool_name: Option<String>,
    #[serde(default)]
    pub(crate) args: HashMap<String, ArgDef>,
}

/// Parameter definition declared inside a skill header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArgDef {
    #[serde(rename = "type")]
    pub(crate) arg_type: String,
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) required: bool,
}

/// Runtime skill model stored in the local index and rendered by the CLI/TUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Skill {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    pub(crate) command_template: Option<String>,
    pub(crate) parameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) tool_name: Option<String>,
}

impl From<(SkillHeader, String)> for Skill {
    fn from((header, path): (SkillHeader, String)) -> Self {
        let parameters = (!header.args.is_empty()).then(|| build_parameters_schema(&header.args));

        Self {
            name: header.name,
            description: header.description,
            path,
            tags: header.tags,
            command_template: header.command_template,
            parameters,
            checksum: None,
            tool_name: header.tool_name,
        }
    }
}

impl Skill {
    /// Return the stable tool/function name used for agent JSON output.
    pub(crate) fn agent_tool_name(&self) -> String {
        self.tool_name.clone().unwrap_or_else(|| {
            build_auto_tool_name(&self.name, &self.path, self.path_file_stem().as_deref())
        })
    }

    pub(crate) fn path_file_stem(&self) -> Option<String> {
        Path::new(&self.path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(ToOwned::to_owned)
    }
}

fn build_parameters_schema(args: &HashMap<String, ArgDef>) -> serde_json::Value {
    let mut sorted_args: Vec<_> = args.iter().collect();
    sorted_args.sort_by(|(left, _), (right, _)| left.cmp(right));

    let mut properties = serde_json::Map::with_capacity(sorted_args.len());
    let mut required = Vec::new();

    for (name, definition) in sorted_args {
        properties.insert(
            name.clone(),
            serde_json::json!({
                "type": definition.arg_type,
                "description": definition.description
            }),
        );

        if definition.required {
            required.push(name.clone());
        }
    }

    serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}
