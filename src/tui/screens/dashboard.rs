use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Sparkline, Wrap},
};

use crate::{
    app::AppState,
    model::MetricKind,
    tui::{
        layout, theme,
        widgets::{render_header, render_overview},
    },
};

pub fn render_dashboard(frame: &mut Frame, state: &AppState) {
    let chunks = layout::dashboard_chunks(frame.size());
    render_header(frame, chunks[0], state);
    render_overview(frame, chunks[1], state);
    let body = Block::default().title("CPU / Memory history").borders(Borders::ALL);
    let body_inner = body.inner(chunks[2]);
    frame.render_widget(body, chunks[2]);
    let spark_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(body_inner);
    let cpu = spark_values(state.histories.get(&MetricKind::Cpu));
    let memory = spark_values(state.histories.get(&MetricKind::Memory));
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("CPU %"))
            .data(&cpu)
            .style(theme::gauge()),
        spark_chunks[0],
    );
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("Memory %"))
            .data(&memory)
            .style(theme::warning()),
        spark_chunks[1],
    );

    let footer = if state.help_visible {
        "q quit | 1 dashboard | 2 processes | 3 alerts | 4 history | s record | space pause | R reset | ? close help"
    } else {
        "q quit | 1 dashboard | 2 processes | 3 alerts | 4 history | s record | space pause | ? help"
    };
    frame.render_widget(Paragraph::new(footer).style(theme::muted()), chunks[3]);
    if state.help_visible {
        let area = centered_rect(70, 50, frame.size());
        frame.render_widget(Paragraph::new("rstats controls\n\nq / Ctrl-C  Quit\n1 / 2 / 3 / 4 Views\ns           Start/stop recording\nspace       Pause updates\nR           Reset history\nc / m       Sort processes\nr           Reverse sort\n?           Toggle help").block(Block::default().title("Help").borders(Borders::ALL)).wrap(Wrap { trim: true }), area);
    }
}

fn spark_values(history: Option<&crate::history::History>) -> Vec<u64> {
    history
        .map(|history| history.values().map(|value| value.clamp(0.0, 100.0) as u64).collect())
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
