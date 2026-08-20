use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Sparkline, Wrap},
};

use crate::{
    app::AppState,
    model::MetricKind,
    tui::{theme, widgets::render_header},
};

pub fn render_history_screen(frame: &mut Frame, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10), Constraint::Length(2)])
        .split(frame.size());
    render_header(frame, chunks[0], state);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);
    let items = if state.saved_recordings.is_empty() {
        vec![ListItem::new("No saved recordings")]
    } else {
        state
            .saved_recordings
            .iter()
            .map(|session| {
                let name =
                    session.path.file_name().and_then(|name| name.to_str()).unwrap_or("recording");
                ListItem::new(format!("{name}  ({} samples)", session.samples))
            })
            .collect()
    };
    let list = List::new(items)
        .block(Block::default().title("Saved recordings").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut list_state = ratatui::widgets::ListState::default();
    if !state.saved_recordings.is_empty() {
        list_state.select(Some(state.selected_recording));
    }
    frame.render_stateful_widget(list, columns[0], &mut list_state);

    let session = if state.recorder.is_some() { None } else { state.loaded_recording.as_ref() };
    let status = if let Some(recorder) = state.recorder.as_ref() {
        format!(
            "Recording now\n{}\nSamples: {}",
            recorder.path().display(),
            recorder.sample_count()
        )
    } else if let Some(session) = session {
        format!(
            "Loaded: {}\nSamples: {} | Duration: {}s",
            session.path.display(),
            session.samples,
            session.duration_seconds()
        )
    } else if let Some(summary) = state.last_recording.as_ref() {
        format!(
            "Last recording: {}\nSamples: {} | Duration: {}s\nPress Enter to load it",
            summary.path.display(),
            summary.samples,
            summary.duration_seconds()
        )
    } else {
        "Select a saved recording and press Enter\nPress s to start a new recording".to_owned()
    };
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(8)])
        .split(columns[1]);
    frame.render_widget(
        Paragraph::new(status)
            .block(Block::default().title("Recording details").borders(Borders::ALL))
            .wrap(Wrap { trim: true }),
        right[0],
    );

    let chart = Block::default().title("Recorded history").borders(Borders::ALL);
    let inner = chart.inner(right[1]);
    frame.render_widget(chart, right[1]);
    let spark_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);
    let cpu = session
        .map_or_else(|| current_values(state, MetricKind::Cpu), |session| session.cpu.clone());
    let memory = session.map_or_else(
        || current_values(state, MetricKind::Memory),
        |session| session.memory.clone(),
    );
    let cpu = spark_values(&cpu);
    let memory = spark_values(&memory);
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

    let footer = state
        .recording_error
        .as_deref()
        .map_or("Up/Down select | Enter load | s start/stop | 1 dashboard | q quit", |error| error);
    frame.render_widget(Paragraph::new(footer).style(theme::muted()), chunks[2]);
}

fn current_values(state: &AppState, metric: MetricKind) -> Vec<f64> {
    state
        .recording_histories
        .get(&metric)
        .map(|history| history.values().copied().collect())
        .unwrap_or_default()
}

fn spark_values(values: &[f64]) -> Vec<u64> {
    values.iter().map(|value| value.clamp(0.0, 100.0) as u64).collect()
}
