mod highlight;
mod hit_test;
mod render;
mod state;
mod view_model;

use crate::domain::Skill;
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseButton, MouseEventKind,
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
            Event::Key(key) => {
                if let Some(selection) = handle_key_event(app, key) {
                    return Ok(selection);
                }
            }
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

fn handle_key_event(app: &mut App, key: KeyEvent) -> Option<Option<Skill>> {
    // Windows 下 crossterm 会额外发出 Release/Repeat 事件。
    // 这里只接受 Press，避免刚进入 TUI 时残留的 Enter 释放事件被误判为“立即确认选择”。
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Char(character) => {
            app.search_input.push(character);
            app.filter_items();
            None
        }
        KeyCode::Backspace => {
            app.search_input.pop();
            app.filter_items();
            None
        }
        KeyCode::Enter => Some(app.selected_skill().cloned()),
        KeyCode::Esc => Some(None),
        KeyCode::Up => {
            app.previous();
            None
        }
        KeyCode::Down => {
            app.next();
            None
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_skill(name: &str) -> Skill {
        Skill {
            name: name.to_owned(),
            description: format!("{name} description"),
            path: format!("/tmp/{name}.md"),
            tags: Vec::new(),
            command_template: None,
            parameters: None,
            checksum: None,
            tool_name: None,
        }
    }

    fn key_event(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent::new_with_kind(code, crossterm::event::KeyModifiers::NONE, kind)
    }

    #[test]
    fn test_enter_release_does_not_select_current_item() {
        let mut app = App::new(vec![sample_skill("alpha")]);
        app.filter_items();

        let selection =
            handle_key_event(&mut app, key_event(KeyCode::Enter, KeyEventKind::Release));

        // Windows 会在按键释放阶段补发事件，这里必须忽略，避免 TUI 一启动就直接确认选中。
        assert!(selection.is_none());
        assert_eq!(
            app.selected_skill().map(|skill| skill.name.as_str()),
            Some("alpha")
        );
    }

    #[test]
    fn test_enter_press_selects_current_item() {
        let mut app = App::new(vec![sample_skill("alpha")]);
        app.filter_items();

        let selection = handle_key_event(&mut app, key_event(KeyCode::Enter, KeyEventKind::Press));

        // 真正的 Press 事件仍然要正常返回当前命中的技能，不能把确认逻辑一并拦掉。
        assert_eq!(
            selection.and_then(|skill| skill).map(|skill| skill.name),
            Some(String::from("alpha"))
        );
    }
}
