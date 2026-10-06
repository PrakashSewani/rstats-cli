use crate::{app::AppState, format::format_bytes, tui::theme};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

pub fn render_processes(frame: &mut Frame, area: Rect, state: &AppState) {
    let processes = state.process_view.visible(&state.snapshot.processes);
    let rows = processes.iter().map(|process| {
        Row::new(vec![
            Cell::from(process.pid.to_string()),
            Cell::from(process.cpu_usage.to_string()),
            Cell::from(format_bytes(process.memory_bytes)),
            Cell::from(process.user.clone()),
            Cell::from(if process.command.is_empty() {
                process.name.clone()
            } else {
                process.command.clone()
            }),
        ])
    });
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Min(20),
        ],
    )
    .header(Row::new(vec!["PID", "CPU %", "MEM", "USER", "COMMAND"]).style(theme::title()))
    .block(Block::default().title(format!("Processes ({})", processes.len())).borders(Borders::ALL))
    .column_spacing(1)
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(table, area, &mut TableState::default());
}
