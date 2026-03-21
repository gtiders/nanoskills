use crate::domain::Skill;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use rust_i18n::t;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

fn select_theme(theme_set: &ThemeSet) -> Option<&Theme> {
    theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.values().next())
}

pub(super) fn highlight_text(text: &str, indices: &[usize]) -> Line<'static> {
    if indices.is_empty() {
        return Line::from(text.to_string());
    }

    let chars: Vec<char> = text.chars().collect();
    let mut sorted_indices = indices.to_vec();
    sorted_indices.sort_unstable();
    sorted_indices.dedup();

    let mut spans = Vec::new();
    let mut current = 0;

    for &index in &sorted_indices {
        if index >= chars.len() {
            continue;
        }

        if current < index {
            spans.push(Span::raw(chars[current..index].iter().collect::<String>()));
        }

        spans.push(Span::styled(
            chars[index].to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        current = index + 1;
    }

    if current < chars.len() {
        spans.push(Span::raw(chars[current..].iter().collect::<String>()));
    }

    Line::from(spans)
}

pub(super) fn build_preview_lines(skill: &Skill) -> Vec<Line<'static>> {
    match serde_yaml::to_string(skill) {
        Ok(yaml_content) => highlight_yaml_content(&yaml_content),
        Err(error) => vec![Line::from(Span::styled(
            t!("ui.preview_render_failed", reason = error).to_string(),
            Style::default().fg(Color::Red),
        ))],
    }
}

fn highlight_yaml_content(yaml_content: &str) -> Vec<Line<'static>> {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();
    let syntax = syntax_set
        .find_syntax_by_extension("yaml")
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let Some(theme) = select_theme(theme_set) else {
        return yaml_content
            .lines()
            .map(|line| Line::from(line.to_string()))
            .collect();
    };

    let mut highlighter = HighlightLines::new(syntax, theme);

    LinesWithEndings::from(yaml_content)
        .map(|line| match highlighter.highlight_line(line, syntax_set) {
            Ok(ranges) => {
                let spans: Vec<_> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let color =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        Span::styled(text.to_string(), Style::default().fg(color))
                    })
                    .collect();
                Line::from(spans)
            }
            Err(_) => Line::from(line.to_string()),
        })
        .collect()
}
