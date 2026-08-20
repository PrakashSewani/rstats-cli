use crate::{
    app::AppState,
    tui::{layout, theme},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge},
};

pub fn render_overview(frame: &mut Frame, area: Rect, state: &AppState) {
    let columns = layout::overview_chunks(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(columns[1]);
    gauge(frame, left[0], "CPU", state.snapshot.cpu.total_usage, theme::gauge());
    gauge(frame, left[1], "Memory", state.snapshot.memory.used_percent, theme::warning());
    gauge(frame, right[0], "Swap", state.snapshot.swap.used_percent, theme::warning());
    gauge(
        frame,
        right[1],
        "Alerts",
        state.alerts.len() as f64,
        if state.alerts.is_empty() { theme::gauge() } else { theme::critical() },
    );
}

fn gauge(frame: &mut Frame, area: Rect, label: &str, value: f64, style: Style) {
    let value = value.clamp(0.0, 100.0);
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title(label).borders(Borders::ALL))
            .gauge_style(style)
            .percent(value as u16)
            .label(format!("{value:.1}%")),
        area,
    );
}
