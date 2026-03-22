use crate::domain::Skill;
use anyhow::Result;
use crossterm::{
    QueueableCommand,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use rust_i18n::t;
use serde::Serialize;
use std::io::{IsTerminal, Write};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

pub(crate) fn print_skill_yaml_highlighted(skill: &Skill) -> Result<()> {
    print_serialized_highlighted(skill, "yaml", "YAML")
}

pub(crate) fn print_json_highlighted<T>(value: &T, label: &str) -> Result<()>
where
    T: Serialize,
{
    print_serialized_highlighted_with(
        || serde_json::to_string_pretty(value).map_err(|error| error.to_string()),
        "json",
        label,
    )
}

fn select_theme(theme_set: &ThemeSet) -> Option<&Theme> {
    theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.values().next())
}

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn print_serialized_highlighted<T>(value: &T, extension: &str, label: &str) -> Result<()>
where
    T: Serialize,
{
    print_serialized_highlighted_with(
        || serde_yaml::to_string(value).map_err(|error| error.to_string()),
        extension,
        label,
    )
}

fn print_serialized_highlighted_with<F>(serialize: F, extension: &str, label: &str) -> Result<()>
where
    F: FnOnce() -> std::result::Result<String, String>,
{
    let content = match serialize() {
        Ok(content) => content,
        Err(error) => {
            println!("{}", t!("cli.render_failed", label = label, reason = error));
            return Ok(());
        }
    };

    // `--json` 既要支持终端阅读，也要保持被管道和测试消费时是合法原始 JSON。
    // 非 TTY 场景下直接输出纯文本，避免 ANSI 转义序列破坏 Agent 契约。
    if !std::io::stdout().is_terminal() {
        print!("{content}");
        return Ok(());
    }

    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();
    let syntax = syntax_set
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let Some(theme) = select_theme(theme_set) else {
        print!("{content}");
        return Ok(());
    };

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut stdout = std::io::stdout();
    for line in LinesWithEndings::from(&content) {
        match highlighter.highlight_line(line, syntax_set) {
            Ok(ranges) => {
                for (style, text) in ranges {
                    // CLI 输出需要保留 syntect 的分段着色结果，逐段写入终端可以避免把整行重新拼接后丢失颜色边界。
                    stdout.queue(SetForegroundColor(Color::Rgb {
                        r: style.foreground.r,
                        g: style.foreground.g,
                        b: style.foreground.b,
                    }))?;
                    stdout.queue(Print(text))?;
                }
            }
            Err(_) => {
                stdout.queue(Print(line))?;
            }
        }
    }

    stdout.queue(ResetColor)?;
    stdout.flush()?;
    Ok(())
}
