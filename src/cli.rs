use crate::cmd_sync::{SkillSearcher, load_index, print_sync_result, run_sync};
use crate::config::{init_config, resolve_config};
use crate::models::{OpenAITool, Skill};
use crate::ui::run_tui;
use anyhow::Result;
use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nanoskills")]
#[command(about = "Agent 本地技能库 CLI - 极速、零配置的技能管理工具")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    #[command(about = "初始化配置文件")]
    Init {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'f', long, help = "强制覆盖已存在的配置文件")]
        force: bool,
    },

    #[command(about = "同步/构建索引")]
    Sync {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(long, help = "严格模式：显示解析失败的文件")]
        strict: bool,
    },

    #[command(about = "列出所有技能")]
    List {
        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'j', long, help = "输出 JSON 格式（Agent 模式）")]
        json: bool,

        #[arg(short, long, help = "显示详细信息")]
        detailed: bool,
    },

    #[command(about = "交互式选择技能 (TUI)")]
    Pick {
        #[arg(short, long, default_value = ".")]
        path: String,
    },

    #[command(about = "搜索技能 (模糊匹配)")]
    Search {
        #[arg(required = true)]
        query: String,

        #[arg(short = 'j', long, help = "输出 OpenAI Tools JSON 格式")]
        json: bool,

        #[arg(short = 'l', long, help = "限制输出数量")]
        limit: Option<usize>,
    },

    #[command(about = "查看技能详情 (严格名称匹配)")]
    Info {
        #[arg(required = true)]
        name: String,

        #[arg(short = 'j', long, help = "输出 JSON 格式")]
        json: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path, force } => {
            let path_buf = PathBuf::from(&path);
            match init_config(&path_buf, force) {
                Ok(config) => {
                    println!("✓ 配置文件已创建: {}/.agent-skills.yaml", path);
                    println!("扫描路径: {:?}", config.scan_paths);
                    println!("最大文件大小: {}", config.max_file_size);
                    println!("搜索结果限制: {}", config.search_limit);
                }
                Err(e) => {
                    eprintln!("❌ {}", e);
                }
            }
        }

        Commands::Sync { path, strict } => {
            let path_buf = PathBuf::from(&path);
            let result = run_sync(&path_buf, strict)?;
            print_sync_result(&result);
        }

        Commands::List {
            path: _,
            json,
            detailed,
        } => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("索引不存在，请先运行 'nanoskills sync'");
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
                print_simple_table(&index.skills);
            }
        }

        Commands::Pick { path: _ } => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("索引不存在，请先运行 'nanoskills sync'");
                    return Ok(());
                }
            };

            match run_tui(index.skills)? {
                Some(skill) => {
                    print_skill_yaml_highlighted(&skill);
                    println!("\n📁 路径: {}", skill.path);
                }
                None => {
                    eprintln!("未选择任何技能");
                }
            }
        }

        Commands::Search { query, json, limit } => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("索引不存在，请先运行 'nanoskills sync'");
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
                    println!("未找到匹配的技能");
                    return Ok(());
                }

                println!(
                    "找到 {} 个技能 (显示前 {} 个):\n",
                    limited_results.len(),
                    search_limit
                );
                for (skill, score) in &limited_results {
                    println!("  {} - {} [得分: {}]", skill.name, skill.description, score);
                    if !skill.tags.is_empty() {
                        println!("    标签: {}", skill.tags.join(", "));
                    }
                    println!("    路径: {}", skill.path);
                    println!();
                }
            }
        }

        Commands::Info { name, json } => {
            let index = match load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("索引不存在，请先运行 'nanoskills sync'");
                    return Ok(());
                }
            };

            let searcher = SkillSearcher::new(index);

            match searcher.get_by_name(&name) {
                Some(skill) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&skill)?);
                    } else {
                        print_skill_yaml_highlighted(skill);
                    }
                }
                None => {
                    eprintln!("未找到技能: {}", name);
                }
            }
        }
    }

    Ok(())
}

fn print_simple_table(skills: &[Skill]) {
    if skills.is_empty() {
        println!("暂无技能");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec!["序号", "名称", "描述", "标签"]);

    for (i, skill) in skills.iter().enumerate() {
        let tags = skill.tags.join(", ");
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(&skill.name),
            Cell::new(&skill.description),
            Cell::new(tags),
        ]);
    }

    println!("{table}");
    println!("\n共 {} 个技能", skills.len());
}

fn print_detailed_table(skills: &[Skill]) {
    if skills.is_empty() {
        println!("暂无技能");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec!["序号", "名称", "描述", "标签", "参数定义"]);

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
    println!("\n共 {} 个技能", skills.len());
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
