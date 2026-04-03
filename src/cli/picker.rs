use crate::model::Skill;
use anyhow::{Result, anyhow};
use skim::prelude::*;
use std::borrow::Cow;
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

pub(crate) fn run_skim_picker(items: Vec<Skill>) -> Result<Option<Skill>> {
    if items.is_empty() {
        return Ok(None);
    }

    let options = build_options()?;
    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    let entries = items
        .into_iter()
        .map(PickerItem::new)
        .map(|item| Arc::new(item) as Arc<dyn SkimItem>)
        .collect::<Vec<_>>();

    tx.send(entries)?;
    drop(tx);

    let output = Skim::run_with(options, Some(rx)).map_err(|error| anyhow!(error.to_string()))?;

    let selected = output
        .selected_items
        .into_iter()
        .next()
        .and_then(|item| item.downcast_item::<PickerItem>().cloned())
        .map(|item| item.skill);

    Ok(selected)
}

fn build_options() -> Result<SkimOptions> {
    Ok(SkimOptionsBuilder::default()
        .height("100%")
        .highlight_line(true)
        .preview("")
        .multi(false)
        .prompt("🔎 ")
        .preview_window("right:35%:wrap")
        .build()?)
}

#[derive(Clone)]
struct PickerItem {
    skill: Skill,
    search_text: String,
}

impl PickerItem {
    fn new(skill: Skill) -> Self {
        let search_text = format!(
            "{description}",
            description = skill.description
        );

        Self { skill, search_text }
    }
}

impl SkimItem for PickerItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.search_text)
    }

    fn output(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.skill.path)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        ItemPreview::AnsiText(render_preview(&self.skill))
    }
}

fn render_preview(skill: &Skill) -> String {
    let yaml = match serde_yaml::to_string(skill) {
        Ok(content) => content,
        Err(error) => {
            return format!("render failed: {error}\npath: {}", skill.path);
        }
    };

    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set
        .find_syntax_by_extension("yaml")
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let Some(theme) = select_theme(&theme_set) else {
        return yaml;
    };

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut rendered = String::new();
    for line in LinesWithEndings::from(&yaml) {
        match highlighter.highlight_line(line, &syntax_set) {
            Ok(ranges) => {
                let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], false);
                rendered.push_str(&escaped);
            }
            Err(_) => rendered.push_str(line),
        }
    }

    rendered
}

fn select_theme(theme_set: &ThemeSet) -> Option<&Theme> {
    theme_set
        .themes
        .get("base16-ocean.dark")
        .or_else(|| theme_set.themes.values().next())
}
