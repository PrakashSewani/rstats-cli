use crate::{app::AppState, tui::theme};
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

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KiB", "MiB", "GiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}
