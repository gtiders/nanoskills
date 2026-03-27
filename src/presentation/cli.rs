use crate::app::SkillEngine;
use crate::presentation::commands;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nanoskills")]
#[command(about = "为 AI Agent 构建极速本地技能库索引与检索 CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a local configuration file")]
    Init {
        #[arg(short = 'f', long, help = "Overwrite the existing configuration file")]
        force: bool,

        #[arg(long, help = "Create the configuration file in the current directory")]
        local: bool,
    },

    #[command(about = "Build the local skill index")]
    Sync {
        #[arg(long, help = "Fail on invalid skill headers and report parse errors")]
        strict: bool,
    },

    #[command(about = "Print default/local/effective configs")]
    Config,

    #[command(about = "List indexed skills")]
    List {
        #[arg(short = 'j', long, help = "Print machine-readable JSON")]
        json: bool,

        #[arg(short, long, help = "Show full skill details")]
        detailed: bool,
    },

    #[command(about = "Browse and pick a skill in the TUI")]
    Pick,

    #[command(about = "Search indexed skills with fuzzy matching")]
    Search {
        #[arg(required = true)]
        query: String,

        #[arg(short = 'j', long, help = "Print OpenAI tool-call JSON")]
        json: bool,

        #[arg(short = 'l', long, help = "Limit the number of results")]
        limit: Option<usize>,
    },
}

/// Parse CLI args and dispatch the selected command.
pub(crate) fn run() -> Result<()> {
    let engine = SkillEngine::new();

    match Cli::parse().command {
        None => commands::run_default_command(&engine),
        Some(Commands::Init { force, local }) => commands::run_init(&engine, force, local),
        Some(Commands::Sync { strict }) => commands::run_sync(&engine, strict),
        Some(Commands::Config) => commands::run_config(&engine),
        Some(Commands::List { json, detailed }) => commands::run_list(&engine, json, detailed),
        Some(Commands::Pick) => commands::run_pick(&engine),
        Some(Commands::Search { query, json, limit }) => {
            commands::run_search(&engine, &query, json, limit)
        }
    }
}
