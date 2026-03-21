mod highlight;
mod hit_test;
mod render;
mod state;
mod view_model;

use crate::domain::Skill;
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use self::state::App;

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn enter() -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
            let _ = disable_raw_mode();
            return Err(error.into());
        }

        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }

    fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<io::Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

/// Launch the full-screen TUI picker and return the selected skill.
pub(crate) fn run_tui(items: Vec<Skill>) -> Result<Option<Skill>> {
    let mut session = TerminalSession::enter()?;
    let mut app = App::new(items);
    app.filter_items();

    run_app(session.terminal_mut(), &mut app)
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<Option<Skill>> {
    loop {
        terminal.draw(|frame| render::render(frame, app))?;

        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char(character) => {
                    app.search_input.push(character);
                    app.filter_items();
                }
                KeyCode::Backspace => {
                    app.search_input.pop();
                    app.filter_items();
                }
                KeyCode::Enter => return Ok(app.selected_skill().cloned()),
                KeyCode::Esc => return Ok(None),
                KeyCode::Up => app.previous(),
                KeyCode::Down => app.next(),
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => app.previous(),
                MouseEventKind::ScrollDown => app.next(),
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(index) = hit_test::calculate_clicked_index(
                        mouse.row,
                        app.table_area,
                        &app.state,
                        app.filtered_items.len(),
                    ) {
                        app.select_index(index);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
