use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Sparkline, Wrap},
};

use crate::{
    app::AppState,
    model::MetricKind,
    tui::{
        layout, theme,
        widgets::{percent_values, rate_values, render_header, render_overview, render_storage},
    },
};

pub fn render_dashboard(frame: &mut Frame, state: &AppState) {
    let dashboard = layout::dashboard_chunks(frame.size(), state.snapshot.disks.len());
    render_header(frame, dashboard.header, state);
    render_overview(frame, dashboard.overview, state);
    if let Some(storage) = dashboard.storage {
        render_storage(frame, storage, &state.snapshot.disks);
    }
    let body = Block::default().title("History").borders(Borders::ALL);
    let body_inner = body.inner(dashboard.history);
    frame.render_widget(body, dashboard.history);
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body_inner);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(row_chunks[0]);
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(row_chunks[1]);
    let cpu = percent_values(&history_values(state, MetricKind::Cpu));
    let memory = percent_values(&history_values(state, MetricKind::Memory));
    let net = rate_values(
        &history_values(state, MetricKind::NetworkReceive),
        &history_values(state, MetricKind::NetworkTransmit),
    );
    let disk = percent_values(&history_values(state, MetricKind::Disk));
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("CPU %"))
            .data(&cpu)
            .style(theme::gauge()),
        top[0],
    );
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("Memory %"))
            .data(&memory)
            .style(theme::warning()),
        top[1],
    );
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("Net I/O /s"))
            .data(&net)
            .style(theme::title()),
        bottom[0],
    );
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("Disk %"))
            .data(&disk)
            .style(theme::critical()),
        bottom[1],
    );

    let recording_action = state.recording_action();
    let footer = if state.help_visible {
        format!(
            "q quit | 1 dashboard | 2 processes | 3 alerts | 4 history | {recording_action} | space pause | R reset | ? close help"
        )
    } else {
        format!(
            "q quit | 1 dashboard | 2 processes | 3 alerts | 4 history | {recording_action} | space pause | ? help"
        )
    };
    frame.render_widget(Paragraph::new(footer).style(theme::muted()), dashboard.footer);
    if state.help_visible {
        let area = centered_rect(70, 50, frame.size());
        frame.render_widget(Paragraph::new("rstats controls\n\nq / Ctrl-C  Quit\n1 / 2 / 3 / 4 Views\ns           Start/stop recording\nS           Toggle deep capture\nspace       Pause updates\nR           Reset history\nc / m       Sort processes\nr           Reverse sort\nt           Cycle color theme\n?           Toggle help").block(Block::default().title("Help").borders(Borders::ALL)).wrap(Wrap { trim: true }), area);
    }
}

fn history_values(state: &AppState, kind: MetricKind) -> Vec<f64> {
    state
        .histories
        .get(&kind)
        .map(|history| history.values().copied().collect())
        .unwrap_or_default()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
