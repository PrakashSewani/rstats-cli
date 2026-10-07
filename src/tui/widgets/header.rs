use crate::{app::AppState, recording::CaptureScope, tui::theme};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub fn render_header(frame: &mut Frame, area: Rect, state: &AppState) {
    let hostname = state.snapshot.hostname.as_deref().unwrap_or("unknown host");
    let os = state.snapshot.os.as_deref().unwrap_or("unknown OS");
    let status = if state.paused { "PAUSED" } else { "LIVE" };
    let recording = state.recorder.as_ref().map_or_else(String::new, |recorder| {
        let scope = if state.record_scope == CaptureScope::Deep { " · deep" } else { "" };
        format!("  REC [{} samples{scope}]", recorder.sample_count())
    });
    let error = state.collector_error.as_deref().unwrap_or("");
    let text = format!(
        "rstats  {status}{recording}  {hostname}  {os}  uptime {}s  {}",
        state.snapshot.uptime_secs, error
    );
    frame.render_widget(
        Paragraph::new(text).style(theme::title()).block(Block::default().borders(Borders::ALL)),
        area,
    );
}
