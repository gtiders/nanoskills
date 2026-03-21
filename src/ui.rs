use crate::models::Skill;
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
};
use rust_i18n::t;
use std::io;
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
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

pub struct App {
    items: Vec<Skill>,
    filtered_items: Vec<(Skill, i64, Vec<usize>)>,
    state: TableState,
    search_input: String,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
    last_search: String,
    table_area: Rect,
}

impl App {
    pub fn new(items: Vec<Skill>) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        Self {
            items: items.clone(),
            filtered_items: items.iter().map(|s| (s.clone(), 0, Vec::new())).collect(),
            state,
            search_input: String::new(),
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
            last_search: String::new(),
            table_area: Rect::default(),
        }
    }

    pub fn filter_items(&mut self) {
        if self.search_input == self.last_search {
            return;
        }

        self.filtered_items.clear();
        let query = &self.search_input;

        if query.is_empty() {
            self.filtered_items = self
                .items
                .iter()
                .map(|skill| (skill.clone(), 0, Vec::new()))
                .collect();
        } else {
            self.filtered_items = self
                .items
                .iter()
                .filter_map(|skill| {
                    let combined = format!(
                        "{} {} {} {}",
                        skill.name,
                        skill.description,
                        skill.tags.join(" "),
                        skill.path
                    );
                    self.matcher
                        .fuzzy_indices(&combined, query)
                        .map(|(score, indices)| (skill.clone(), score, indices))
                })
                .collect();

            self.filtered_items.sort_by(|a, b| b.1.cmp(&a.1));
        }

        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }

        self.last_search = self.search_input.clone();
    }

    pub fn next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select_index(&mut self, index: usize) {
        if index < self.filtered_items.len() {
            self.state.select(Some(index));
        }
    }

    pub fn selected_skill(&self) -> Option<&Skill> {
        self.state
            .selected()
            .and_then(|i| self.filtered_items.get(i))
            .map(|(skill, _, _)| skill)
    }
}

fn highlight_text(s: &str, indices: &[usize]) -> Line<'static> {
    if indices.is_empty() {
        return Line::from(s.to_string());
    }

    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = s.chars().collect();

    let mut sorted_indices = indices.to_vec();
    sorted_indices.sort();
    sorted_indices.dedup();

    for &idx in &sorted_indices {
        if idx >= chars.len() {
            continue;
        }

        if current_pos < idx {
            spans.push(Span::styled(
                chars[current_pos..idx].iter().collect::<String>(),
                Style::default(),
            ));
        }

        spans.push(Span::styled(
            chars[idx].to_string(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));

        current_pos = idx + 1;
    }

    if current_pos < chars.len() {
        spans.push(Span::styled(
            chars[current_pos..].iter().collect::<String>(),
            Style::default(),
        ));
    }

    Line::from(spans)
}

fn highlight_yaml_content(yaml_content: &str) -> Vec<Line<'static>> {
    let syntax_set = get_syntax_set();
    let theme_set = get_theme_set();

    let syntax = syntax_set
        .find_syntax_by_extension("yaml")
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    let theme = &theme_set.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for line in LinesWithEndings::from(yaml_content) {
        let ranges: Vec<(syntect::highlighting::Style, &str)> =
            h.highlight_line(line, syntax_set).unwrap_or_default();

        let spans: Vec<Span> = ranges
            .into_iter()
            .map(|(style, text)| {
                let color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                Span::styled(text.to_string(), Style::default().fg(color))
            })
            .collect();

        lines.push(Line::from(spans));
    }

    lines
}

fn build_preview_lines(skill: &Skill) -> Vec<Line<'static>> {
    match serde_yaml::to_string(skill) {
        Ok(yaml_content) => highlight_yaml_content(&yaml_content),
        Err(e) => {
            vec![Line::from(Span::styled(
                format!("❌ YAML serialization failed: {}", e),
                Style::default().fg(Color::Red),
            ))]
        }
    }
}

fn calculate_clicked_index(
    mouse_y: u16,
    table_rect: Rect,
    table_state: &TableState,
    item_count: usize,
) -> Option<usize> {
    let table_top = table_rect.top();
    let table_bottom = table_rect.bottom();

    if mouse_y < table_top || mouse_y >= table_bottom {
        return None;
    }

    let header_height = 1u16;
    let border_height = 1u16;

    if mouse_y < table_top + border_height + header_height {
        return None;
    }

    let clicked_row = (mouse_y - table_top - border_height - header_height) as usize;

    let offset = table_state.offset();
    let absolute_index = offset + clicked_row;

    if absolute_index < item_count {
        Some(absolute_index)
    } else {
        None
    }
}

pub fn run_tui(items: Vec<Skill>) -> Result<Option<Skill>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(items);
    app.filter_items();

    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<Option<Skill>> {
    loop {
        terminal.draw(|f| render(f, app))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => {
                    app.search_input.push(c);
                    app.filter_items();
                }
                KeyCode::Backspace => {
                    app.search_input.pop();
                    app.filter_items();
                }
                KeyCode::Enter => {
                    let result = app
                        .state
                        .selected()
                        .map(|i| app.filtered_items[i].0.clone());
                    return Ok(result);
                }
                KeyCode::Esc => {
                    return Ok(None);
                }
                KeyCode::Up => {
                    app.previous();
                }
                KeyCode::Down => {
                    app.next();
                }
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    app.previous();
                }
                MouseEventKind::ScrollDown => {
                    app.next();
                }
                MouseEventKind::Down(button) => {
                    if button == MouseButton::Left
                        && let Some(index) = calculate_clicked_index(
                            mouse.row,
                            app.table_area,
                            &app.state,
                            app.filtered_items.len(),
                        )
                    {
                        app.select_index(index);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

fn render(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(size);

    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(main_chunks[0]);

    let header = Row::new(vec![
        Cell::from(t!("ui.table_header.index").to_string()).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(t!("ui.table_header.name").to_string()).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(t!("ui.table_header.tags").to_string()).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Cell::from(t!("ui.table_header.description").to_string()).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ])
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(i, (skill, _, indices))| {
            let index_cell = Cell::from(format!("{}", i + 1));

            let name_line = highlight_text(&skill.name, indices);
            let name_cell = Cell::from(name_line);

            let tags_text = if skill.tags.is_empty() {
                "-".to_string()
            } else {
                skill.tags.join(", ")
            };
            let tags_line = highlight_text(&tags_text, indices);
            let tags_cell = Cell::from(tags_line);

            let desc_line = highlight_text(&skill.description, indices);
            let desc_cell = Cell::from(desc_line);

            let is_selected = app.state.selected() == Some(i);
            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![index_cell, name_cell, tags_cell, desc_cell]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(20),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ],
    )
    .header(header)
    .column_spacing(3)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                format!(" {} ", t!("ui.list_title")),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, table_chunks[0], &mut app.state);
    app.table_area = table_chunks[0];

    let preview_lines = if let Some(skill) = app.selected_skill() {
        build_preview_lines(skill)
    } else {
        vec![Line::from(Span::styled(
            t!("ui.no_selection").to_string(),
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let preview_paragraph = Paragraph::new(preview_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    format!(" {} ", t!("ui.preview_title")),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(preview_paragraph, table_chunks[1]);

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            format!(" {} ", t!("ui.search_prompt")),
            Style::default().fg(Color::Green),
        ));
    let search_text = format!("❯ {}█", app.search_input);
    let search_paragraph = Paragraph::new(search_text).block(search_block);
    f.render_widget(search_paragraph, main_chunks[1]);
}
