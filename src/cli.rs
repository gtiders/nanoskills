use crate::cmd_sync::{SkillSearcher, load_index, print_sync_result, run_sync};
use crate::config::{init_config, resolve_config};
use crate::models::{OpenAITool, Skill};
use crate::ui::run_tui;
use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use rust_i18n::t;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nanoskills")]
#[command(about = "Agent 本地技能库 CLI - 极速、零配置的技能管理工具")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "Initialize configuration file")]
    Init {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'f', long, help = "Force overwrite existing configuration")]
        force: bool,
    },

    #[command(about = "Sync and build index")]
    Sync {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(long, help = "Strict mode: show parse errors")]
        strict: bool,
    },

    #[command(about = "List all skills")]
    List {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'j', long, help = "Output in JSON format (Agent mode)")]
        json: bool,

        #[arg(short, long, help = "Show detailed information")]
        detailed: bool,
    },

    #[command(about = "Interactive skill selection (TUI)")]
    Pick {
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    #[command(about = "Search skills (fuzzy match)")]
    Search {
        #[arg(required = true)]
        query: String,

        #[arg(short = 'j', long, help = "Output in OpenAI Tools JSON format")]
        json: bool,

        #[arg(short = 'l', long, help = "Limit number of results")]
        limit: Option<usize>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("{}", t!("cli.index_not_found"));
                    return Ok(());
                }
            };

            match run_tui(index.skills)? {
                Some(skill) => {
                    print_skill_yaml_highlighted(&skill);
                    println!("\n📁 Path: {}", skill.path);
                }
                None => {
                    eprintln!("{}", t!("ui.no_selection"));
                }
            }
        }

        Some(Commands::Init { path, force }) => {
            let path_buf = PathBuf::from(&path);
            match init_config(&path_buf, force) {
                Ok(config) => {
                    println!("{}", t!("cli.config_created", path = path));
                    println!(
                        "{}",
                        t!("cli.scan_paths", paths = format!("{:?}", config.scan_paths))
                    );
                    println!("{}", t!("cli.max_file_size", size = config.max_file_size));
                    println!("{}", t!("cli.search_limit", limit = config.search_limit));
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                }
            }
        }

        Some(Commands::Sync { path, strict }) => {
            let path_buf = PathBuf::from(&path);
            let result = run_sync(&path_buf, strict)?;
            print_sync_result(&result);
        }

        Some(Commands::List {
            path: _,
            json,
            detailed,
        }) => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("{}", t!("cli.index_not_found"));
                    return Ok(());
                }
            };

            if json {
                let json_output: Vec<serde_json::Value> = index
                    .skills
                    .iter()
                    .map(|s| serde_json::to_value(s).unwrap_or(serde_json::json!(null)))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else if detailed {
                print_detailed_table(&index.skills);
            } else {
                print_skills_table(&index.skills);
                println!("{}", t!("cli.total_skills", count = index.skills.len()));
            }
        }

        Some(Commands::Pick { path: _ }) => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("{}", t!("cli.index_not_found"));
                    return Ok(());
                }
            };

            match run_tui(index.skills)? {
                Some(skill) => {
                    print_skill_yaml_highlighted(&skill);
                    println!("\n📁 Path: {}", skill.path);
                }
                None => {
                    eprintln!("{}", t!("ui.no_selection"));
                }
            }
        }

        Some(Commands::Search { query, json, limit }) => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("{}", t!("cli.index_not_found"));
                    return Ok(());
                }
            };

            let config = resolve_config(&std::env::current_dir()?)?;
            let search_limit = limit.unwrap_or(config.search_limit);

            let searcher = SkillSearcher::new(index);
            let results = searcher.fuzzy_search(&query);
            let limited_results: Vec<_> = results.into_iter().take(search_limit).collect();

            if json {
                let tools: Vec<OpenAITool> = limited_results
                    .iter()
                    .map(|(skill, _)| OpenAITool::from(*skill))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&tools)?);
            } else {
                if limited_results.is_empty() {
                    println!("{}", t!("cli.search_not_found"));
                    return Ok(());
                }

                println!(
                    "{}",
                    t!(
                        "cli.search_found",
                        count = limited_results.len(),
                        limit = search_limit
                    )
                );
                let skills: Vec<Skill> = limited_results
                    .iter()
                    .map(|(skill, _)| (*skill).clone())
                    .collect();
                print_skills_table(&skills);
                println!("{}", t!("cli.search_hint"));
            }
        }
    }

    Ok(())
}

fn print_skills_table(skills: &[Skill]) {
    if skills.is_empty() {
        println!("{}", t!("cli.no_skills"));
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        t!("ui.table_header.index"),
        t!("ui.table_header.name"),
        t!("ui.table_header.description"),
        t!("ui.table_header.tags"),
        t!("ui.table_header.path"),
    ]);

    for (i, skill) in skills.iter().enumerate() {
        let tags = skill.tags.join(", ");
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&skill.name),
            Cell::new(&skill.description),
            Cell::new(tags),
            Cell::new(&skill.path),
        ]);
    }

    println!("{table}");
}

fn print_detailed_table(skills: &[Skill]) {
    if skills.is_empty() {
        println!("{}", t!("cli.no_skills"));
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        t!("ui.table_header.index"),
        t!("ui.table_header.name"),
        t!("ui.table_header.description"),
        t!("ui.table_header.tags"),
        t!("ui.table_header.parameters"),
    ]);

    for (i, skill) in skills.iter().enumerate() {
        let tags = skill.tags.join(", ");
        let params_yaml = if let Some(ref params) = skill.parameters {
            serde_yaml::to_string(params).unwrap_or_default()
        } else {
            String::new()
        };

        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&skill.name),
            Cell::new(&skill.description),
            Cell::new(tags),
            Cell::new(params_yaml.trim()),
        ]);
    }

    println!("{table}");
    println!("{}", t!("cli.total_skills", count = skills.len()));
}

fn print_skill_yaml_highlighted(skill: &Skill) {
    let yaml_str = serde_yaml::to_string(skill).unwrap_or_default();

    use std::sync::OnceLock;
    use syntect::easy::HighlightLines;
    use syntect::highlighting::ThemeSet;
    use syntect::parsing::SyntaxSet;
    use syntect::util::LinesWithEndings;

    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

    let syntax_set = SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines);
    let theme_set = THEME_SET.get_or_init(ThemeSet::load_defaults);

    let syntax = syntax_set
        .find_syntax_by_extension("yaml")
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let theme = &theme_set.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(&yaml_str) {
        let ranges: Vec<(syntect::highlighting::Style, &str)> =
            h.highlight_line(line, syntax_set).unwrap_or_default();

        for (style, text) in ranges {
            let color =
                termion::color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
            print!("{}{}", termion::color::Fg(color), text);
        }
    }
    print!("{}", termion::color::Fg(termion::color::Reset));
}
