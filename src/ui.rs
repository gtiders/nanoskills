use crate::models::Skill;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::io;

pub struct App {
    items: Vec<Skill>,
    filtered_items: Vec<(Skill, i64)>,
    state: TableState,
    search_input: String,
    matcher: fuzzy_matcher::skim::SkimMatcherV2,
}

impl App {
    pub fn new(items: Vec<Skill>) -> Self {
        let filtered_items: Vec<(Skill, i64)> = items.iter().map(|s| (s.clone(), 0)).collect();
        let mut state = TableState::default();
        state.select(Some(0));

        App {
            items,
            filtered_items,
            state,
            search_input: String::new(),
            matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
        }
    }

    fn filter(&mut self) {
        if self.search_input.is_empty() {
            self.filtered_items = self.items.iter().map(|s| (s.clone(), 0)).collect();
        } else {
            let mut results: Vec<(Skill, i64)> = self
                .items
                .iter()
                .filter_map(|skill| {
                    let tags_str = skill.tags.join(" ");
                    let haystack = format!(
                        "{} {} {} {}",
                        skill.name, skill.description, tags_str, skill.path
                    );

                    self.matcher
                        .fuzzy_match(&haystack, &self.search_input)
                        .map(|score| (skill.clone(), score))
                })
                .collect();

            results.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_items = results;
        }

        if !self.filtered_items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn selected_path(&self) -> Option<String> {
        self.state
            .selected()
            .and_then(|i| self.filtered_items.get(i).map(|(s, _)| s.path.clone()))
    }
}

pub fn run_tui(items: Vec<Skill>) -> Result<Option<String>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(items);
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(path) => Ok(path),
        Err(err) => {
            eprintln!("Error: {:?}", err);
            Ok(None)
        }
    }
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<Option<String>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    if key.modifiers == KeyModifiers::CONTROL && c == 'c' {
                        return Ok(None);
                    }
                    app.search_input.push(c);
                    app.filter();
                }
                KeyCode::Backspace => {
                    app.search_input.pop();
                    app.filter();
                }
                KeyCode::Up => {
                    app.previous();
                }
                KeyCode::Down => {
                    app.next();
                }
                KeyCode::Enter => {
                    return Ok(app.selected_path());
                }
                KeyCode::Esc => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(f.area());

    let header_cells = ["序号", "名称", "描述", "标签"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
    let header = Row::new(header_cells)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(i, (skill, _))| {
            let tags = skill.tags.join(", ");
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(skill.name.clone()),
                Cell::from(skill.description.clone()),
                Cell::from(tags),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(25),
            Constraint::Percentage(45),
            Constraint::Percentage(20),
        ],
    )
    .header(header)
    .block(
        Block::default().borders(Borders::ALL).title(Span::styled(
            " 技能列表 ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    f.render_stateful_widget(table, chunks[0], &mut app.state.clone());

    let search_block = Block::default().borders(Borders::ALL).title(Span::styled(
        " 模糊搜索 (输入文字 / ↑↓ 选择 / Enter 确认 / Esc 退出) ",
        Style::default().fg(Color::Green),
    ));
    let search_text = format!("> {}█", app.search_input);
    let search_paragraph = Paragraph::new(search_text).block(search_block);
    f.render_widget(search_paragraph, chunks[1]);

    let status_text = format!("共 {} 个技能", app.filtered_items.len());
    let status = Paragraph::new(Span::styled(status_text, Style::default().fg(Color::Gray)));
    let status_area = Layout::default()
        .constraints([Constraint::Length(1)])
        .split(chunks[1])[0];
    f.render_widget(status, status_area);
}
