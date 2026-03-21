use crate::presentation::tui::highlight::{build_preview_lines, highlight_text};
use crate::presentation::tui::state::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Wrap},
};
use rust_i18n::t;

pub(super) fn render(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

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

    let rows: Vec<_> = app
        .filtered_items
        .iter()
        .enumerate()
        .filter_map(|(visible_index, item)| {
            let item_view = app.items.get(item.item_index)?;
            let skill = &item_view.skill;

            let style = if app.state.selected() == Some(visible_index) {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Some(
                Row::new(vec![
                    Cell::from((visible_index + 1).to_string()),
                    Cell::from(highlight_text(&skill.name, &item.highlights.name)),
                    Cell::from(highlight_text(&item_view.tags_text, &item.highlights.tags)),
                    Cell::from(highlight_text(
                        &skill.description,
                        &item.highlights.description,
                    )),
                ])
                .style(style),
            )
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

    frame.render_stateful_widget(table, table_chunks[0], &mut app.state);
    app.table_area = table_chunks[0];

    let preview_lines = app
        .selected_view()
        .map(|item| build_preview_lines(&item.skill))
        .unwrap_or_else(|| {
            vec![Line::from(Span::styled(
                t!("ui.no_selection").to_string(),
                Style::default().fg(Color::DarkGray),
            ))]
        });

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

    frame.render_widget(preview_paragraph, table_chunks[1]);

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            format!(" {} ", t!("ui.search_prompt")),
            Style::default().fg(Color::Green),
        ));

    let search_text = format!("❯ {}█", app.search_input);
    let search_paragraph = Paragraph::new(search_text).block(search_block);
    frame.render_widget(search_paragraph, main_chunks[1]);
}
