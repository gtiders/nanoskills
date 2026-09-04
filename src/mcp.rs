use crate::registry::{MAX_MCP_SEARCH_LIMIT, ScriptName, Skill, load_registry, load_skills};
use crate::search::{script_name, search_skills};
use rmcp::{
    ErrorData, Json, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, Resource, ResourceContents, ServerCapabilities,
        ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
};
use serde::Serialize;
use std::fs;

const INSTRUCTIONS: &str = "This server discovers reusable scripts registered on the user's machine. Search before writing, generating, or executing ad-hoc code or shell commands for any concrete task that could be automated, even when the user does not mention sks, local tools, or existing scripts. This includes calculations, conversions, file and data processing, content generation, validation, and build or development workflows. A request to use a script is always an explicit search trigger. Call search_scripts once with the task's capability and domain terms. Do not search for purely conceptual discussion that needs no execution. If matches are returned, select by description and tags, read the source resource when arguments or behavior are unclear, and use `sks run <name> [args...]` only when shell execution is available and appropriate. Do not invent unsupported arguments. An empty result is normal; continue with another approach without repeatedly retrying equivalent searches.";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub(crate) struct SearchScriptsRequest {
    /// Concise description of the required capability. Optional when tags are provided.
    query: Option<String>,
    /// Optional relevance hints. Matching tags boost ranking but never exclude query matches.
    #[serde(default)]
    tags: Vec<String>,
    /// Maximum results to return. Defaults to 5 and is capped at 10.
    limit: Option<usize>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub(crate) struct SearchScriptsResponse {
    matches: Vec<ScriptMatch>,
    message: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
struct ScriptMatch {
    name: String,
    description: Option<String>,
    tags: Vec<String>,
    usage: String,
    command_template: String,
    details_uri: String,
    source_uri: String,
}

#[derive(Debug, Clone)]
struct SksServer {
    tool_router: ToolRouter<Self>,
}

impl SksServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl SksServer {
    #[tool(
        name = "search_scripts",
        description = "Search and rank reusable scripts registered on the user's machine. Call this once BEFORE writing or running new ad-hoc code or shell commands for an executable task, including calculations, conversions, file/data processing, generation, validation, and build workflows. Do this even if the user did not ask for an existing or local tool; an explicit request to use a script must trigger a search. Query text alone is sufficient and optional tags only improve ranking. If nothing useful matches, continue normally without retrying the same search.",
        annotations(
            title = "Search registered scripts",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    fn search_scripts(
        &self,
        Parameters(request): Parameters<SearchScriptsRequest>,
    ) -> Result<Json<SearchScriptsResponse>, ErrorData> {
        let has_query = request
            .query
            .as_deref()
            .is_some_and(|query| !query.trim().is_empty());
        let tags = request
            .tags
            .into_iter()
            .map(|tag| tag.trim().to_string())
            .filter(|tag| !tag.is_empty())
            .collect::<Vec<_>>();
        if !has_query && tags.is_empty() {
            return Err(ErrorData::invalid_params(
                "provide a non-empty query or at least one tag",
                None,
            ));
        }
        if request
            .limit
            .is_some_and(|limit| !(1..=MAX_MCP_SEARCH_LIMIT).contains(&limit))
        {
            return Err(ErrorData::invalid_params(
                format!("limit must be between 1 and {MAX_MCP_SEARCH_LIMIT}"),
                None,
            ));
        }

        let registry = load_registry().map_err(internal_error)?;
        let limit = request.limit.unwrap_or(registry.mcp_search_limit);
        let matches = search_skills(
            &registry.skills,
            request.query.as_deref(),
            &tags,
            Some(limit),
        )
        .into_iter()
        .map(|matched| script_match(matched.skill))
        .collect::<Vec<_>>();
        let message = if matches.is_empty() {
            "No matching registered scripts found.".to_string()
        } else {
            format!("Found {} matching registered script(s).", matches.len())
        };
        Ok(Json(SearchScriptsResponse { matches, message }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SksServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        let skills = load_skills().map_err(internal_error)?;
        let mut resources = vec![
            Resource::new("sks://registry", "registry")
                .with_title("sks script registry")
                .with_description("A concise index of locally registered scripts")
                .with_mime_type("application/yaml"),
        ];
        for skill in &skills {
            resources.push(
                Resource::new(details_uri(&skill.name), format!("script-{}", skill.name))
                    .with_title(script_name(skill))
                    .with_description(
                        skill
                            .comment
                            .clone()
                            .unwrap_or_else(|| "Registered script metadata".to_string()),
                    )
                    .with_mime_type("application/yaml"),
            );
            resources.push(
                Resource::new(
                    source_uri(&skill.name),
                    format!("script-{}-source", skill.name),
                )
                .with_title(format!("{} source", script_name(skill)))
                .with_description("Source code for the registered script")
                .with_mime_type(source_mime(skill)),
            );
        }
        Ok(ListResourcesResult::with_all_items(resources))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        let skills = load_skills().map_err(internal_error)?;
        let uri = request.uri.as_str();
        let content = if uri == "sks://registry" {
            let summaries = skills.iter().map(script_match).collect::<Vec<_>>();
            ResourceContents::text(
                serde_yaml::to_string(&summaries).map_err(internal_error)?,
                uri,
            )
            .with_mime_type("application/yaml")
        } else if let Some((name, source)) = parse_script_uri(uri) {
            let skill = skills
                .iter()
                .find(|skill| skill.name == name)
                .ok_or_else(|| ErrorData::invalid_params("unknown script resource", None))?;
            if source {
                ResourceContents::text(
                    String::from_utf8_lossy(&fs::read(&skill.path).map_err(internal_error)?)
                        .into_owned(),
                    uri,
                )
                .with_mime_type(source_mime(skill))
            } else {
                ResourceContents::text(
                    serde_yaml::to_string(&script_match(skill)).map_err(internal_error)?,
                    uri,
                )
                .with_mime_type("application/yaml")
            }
        } else {
            return Err(ErrorData::invalid_params("unknown sks resource URI", None));
        };
        Ok(ReadResourceResult::new(vec![content]).into())
    }
}

pub(crate) fn run() -> anyhow::Result<()> {
    tokio::runtime::Runtime::new()?.block_on(async {
        let service = SksServer::new().serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok(())
    })
}

fn script_match(skill: &Skill) -> ScriptMatch {
    ScriptMatch {
        name: skill.name.to_string(),
        description: skill.comment.clone(),
        tags: skill.tags.clone(),
        usage: format!("sks run {} [args...]", skill.name),
        command_template: skill.command.clone(),
        details_uri: details_uri(&skill.name),
        source_uri: source_uri(&skill.name),
    }
}

fn details_uri(name: &ScriptName) -> String {
    format!("sks://scripts/{name}")
}

fn source_uri(name: &ScriptName) -> String {
    format!("sks://scripts/{name}/source")
}

fn parse_script_uri(uri: &str) -> Option<(ScriptName, bool)> {
    let remainder = uri.strip_prefix("sks://scripts/")?;
    let (name, suffix) = remainder.split_once('/').unwrap_or((remainder, ""));
    let name = name.parse().ok()?;
    match suffix {
        "" => Some((name, false)),
        "source" => Some((name, true)),
        _ => None,
    }
}

fn source_mime(skill: &Skill) -> &'static str {
    match skill
        .path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("py") => "text/x-python",
        Some("rs") => "text/x-rust",
        Some("js" | "mjs" | "cjs") => "text/javascript",
        Some("ts" | "mts" | "cts") => "text/typescript",
        Some("sh" | "bash") => "text/x-shellscript",
        Some("ps1") => "text/x-powershell",
        _ => "text/plain",
    }
}

fn internal_error(error: impl std::fmt::Display) -> ErrorData {
    ErrorData::internal_error(error.to_string(), None)
}
