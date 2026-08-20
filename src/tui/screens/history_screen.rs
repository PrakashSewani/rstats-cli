use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Sparkline, Wrap},
};

use crate::{
    app::AppState,
    model::MetricKind,
    tui::{theme, widgets::render_header},
};

pub fn render_history_screen(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(frame.size());
    render_header(frame, chunks[0], state);

    let recording_status = if let Some(recorder) = state.recorder.as_ref() {
        format!(
            "Recording active\n{}\nSamples: {}",
            recorder.path().display(),
            recorder.sample_count()
        )
    } else if let Some(summary) = state.last_recording.as_ref() {
        format!(
            "Last recording\n{}\nSamples: {} | Duration: {}s",
            summary.path.display(),
            summary.samples,
            summary.duration_seconds()
        )
    } else {
        "No recording session yet\nPress s to start recording".to_owned()
    };
    frame.render_widget(
        Paragraph::new(recording_status)
            .block(Block::default().title("Recording session").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        chunks[1],
    );

    let body = Block::default().title("Recorded history").borders(Borders::ALL);
    let inner = body.inner(chunks[2]);
    frame.render_widget(body, chunks[2]);
    let spark_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let cpu = spark_values(state.recording_histories.get(&MetricKind::Cpu));
    let memory = spark_values(state.recording_histories.get(&MetricKind::Memory));
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("Recorded CPU %"))
            .data(&cpu)
            .style(theme::gauge()),
        spark_chunks[0],
    );
    frame.render_widget(
        Sparkline::default()
            .block(Block::default().title("Recorded memory %"))
            .data(&memory)
            .style(theme::warning()),
        spark_chunks[1],
    );
    let footer = state
        .recording_error
        .as_deref()
        .map_or("s start/stop recording | 1 dashboard | 2 processes | 3 alerts", |error| error);
    frame.render_widget(Paragraph::new(footer).style(theme::muted()), chunks[3]);
}

fn spark_values(history: Option<&crate::history::History>) -> Vec<u64> {
    history
        .map(|history| history.values().map(|value| value.clamp(0.0, 100.0) as u64).collect())
        .unwrap_or_default()
}
