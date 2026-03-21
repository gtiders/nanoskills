use crate::domain::Skill;
use serde::Serialize;

/// OpenAI tools-compatible wrapper used by `search --json`.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

/// OpenAI function payload derived from a skill entry.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct OpenAIFunction {
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<serde_json::Value>,
}

impl From<&Skill> for OpenAITool {
    fn from(skill: &Skill) -> Self {
        Self {
            tool_type: "function".to_string(),
            function: OpenAIFunction {
                name: skill.agent_tool_name(),
                description: skill.description.clone(),
                parameters: skill.parameters.clone(),
            },
        }
    }
}
