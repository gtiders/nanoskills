use crate::index::{IndexManager, SkillSearcher, init_config, load_config};
use crate::models::Skill;
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
    },

    #[command(about = "同步/构建索引")]
    Sync {
        #[arg(short, long, default_value = ".")]
        path: String,
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

    #[command(about = "搜索技能")]
    Search {
        #[arg(required = true)]
        query: String,

        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'j', long, help = "输出 JSON 格式（Agent 模式）")]
        json: bool,

        #[arg(short, long, help = "模糊搜索")]
        fuzzy: bool,

        #[arg(short, long, help = "按标签搜索")]
        tags: Vec<String>,
    },

    #[command(about = "查看技能详情")]
    Info {
        #[arg(required = true)]
        name: String,

        #[arg(short, long, default_value = ".")]
        path: String,

        #[arg(short = 'j', long, help = "输出 JSON 格式")]
        json: bool,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let path_buf = PathBuf::from(&path);
            let config = init_config(&path_buf)?;
            println!("✓ 配置文件已创建: {}/.agent-skills.yaml", path);
            println!("扫描路径: {:?}", config.scan_paths);
            println!("文件模式: {:?}", config.file_patterns);
        }

        Commands::Sync { path } => {
            let path_buf = PathBuf::from(&path);
            let config = load_config(&path_buf)?;
            let manager = IndexManager::new(config, &path_buf);
            let index = manager.sync()?;
            println!("✓ 索引已同步: {} 个技能", index.skills.len());
            println!("最后同步时间: {}", index.last_sync);
        }

        Commands::List {
            path,
            json,
            detailed,
        } => {
            let path_buf = PathBuf::from(&path);
            let config = load_config(&path_buf)?;
            let manager = IndexManager::new(config, &path_buf);

            let index = match manager.load_index()? {
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

        Commands::Pick { path } => {
            let path_buf = PathBuf::from(&path);
            let config = load_config(&path_buf)?;
            let manager = IndexManager::new(config, &path_buf);

            let index = match manager.load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("索引不存在，请先运行 'nanoskills sync'");
                    return Ok(());
                }
            };

            match run_tui(index.skills)? {
                Some(selected_path) => {
                    println!("{}", selected_path);
                }
                None => {
                    eprintln!("未选择任何技能");
                }
            }
        }

        Commands::Search {
            query,
            path,
            json,
            fuzzy,
            tags,
        } => {
            let path_buf = PathBuf::from(&path);
            let config = load_config(&path_buf)?;
            let manager = IndexManager::new(config, &path_buf);

            let index = match manager.load_index()? {
                Some(idx) => idx,
                None => {
                    eprintln!("索引不存在，请先运行 'nanoskills sync'");
                    return Ok(());
                }
            };

            let searcher = SkillSearcher::new(index);

            let results: Vec<&Skill> = if !tags.is_empty() {
                searcher.search_by_tags(&tags)
            } else if fuzzy {
                searcher
                    .fuzzy_search(&query)
                    .into_iter()
                    .map(|(s, _)| s)
                    .collect()
            } else {
                searcher.search(&query)
            };

            if json {
                let json_output: Vec<serde_json::Value> = results
                    .iter()
                    .map(|s| serde_json::to_value(s).unwrap_or(serde_json::json!(null)))
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_output)?);
            } else {
                if results.is_empty() {
                    println!("未找到匹配的技能");
                    return Ok(());
                }

                println!("找到 {} 个技能:\n", results.len());
                for skill in results {
                    println!("  {} - {}", skill.name, skill.description);
                    if !skill.tags.is_empty() {
                        println!("    标签: {}", skill.tags.join(", "));
                    }
                    println!("    路径: {}", skill.path);
                    println!();
                }
            }
        }

        Commands::Info { name, path, json } => {
            let path_buf = PathBuf::from(&path);
            let config = load_config(&path_buf)?;
            let manager = IndexManager::new(config, &path_buf);

            let index = match manager.load_index()? {
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
                        print_skill_yaml(skill);
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

fn print_skill_yaml(skill: &Skill) {
    let yaml_str = serde_yaml::to_string(skill).unwrap_or_default();
    println!("{}", yaml_str);
}
