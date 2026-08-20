use crate::{app::AppState, tui::widgets::render_alerts};
use ratatui::prelude::*;

pub fn render_alert_screen(frame: &mut Frame, state: &AppState) {
    render_alerts(frame, frame.size(), state);
}
